use crate::types::LocationError;

#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub bssid: String,
    pub signal_pct: i32,
}

const MAX_ACCESS_POINTS: usize = 12;

pub fn is_scanner_available() -> bool {
    if cfg!(target_os = "windows") {
        std::process::Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        networkkit::Wifi::new().is_available()
    }
}

fn normalize_bssid(raw: &str) -> Option<String> {
    let sep = if raw.contains(':') {
        ':'
    } else if raw.contains('-') {
        '-'
    } else {
        return None;
    };

    let parts: Vec<&str> = raw.split(sep).collect();
    if parts.len() != 6 {
        return None;
    }

    let mut groups = Vec::with_capacity(6);
    for part in parts {
        if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        groups.push(part.to_ascii_lowercase());
    }

    Some(groups.join(":"))
}

fn collect(strongest: &mut std::collections::HashMap<String, AccessPoint>, bssid: String, signal_pct: i32) {
    if !(0..=100).contains(&signal_pct) {
        return;
    }

    match strongest.get_mut(&bssid) {
        Some(entry) => {
            if signal_pct > entry.signal_pct {
                entry.signal_pct = signal_pct;
            }
        }
        None => {
            strongest.insert(bssid.clone(), AccessPoint { bssid, signal_pct });
        }
    }
}

fn finish(strongest: std::collections::HashMap<String, AccessPoint>) -> Vec<AccessPoint> {
    let mut aps: Vec<AccessPoint> = strongest.into_values().collect();
    aps.sort_by(|a, b| b.signal_pct.cmp(&a.signal_pct));
    aps.truncate(MAX_ACCESS_POINTS);
    aps
}

fn parse_netsh_output(stdout: &str) -> Vec<AccessPoint> {
    let mut strongest: std::collections::HashMap<String, AccessPoint> =
        std::collections::HashMap::new();
    let mut current_bssid: Option<String> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();

        if let Some(token) = trimmed.split_whitespace().next_back() {
            if let Some(bssid) = normalize_bssid(token.trim_end_matches(':')) {
                collect(&mut strongest, bssid.clone(), 0);
                current_bssid = Some(bssid);
                continue;
            }
        }

        if trimmed.contains('%') {
            if let Some(pct_token) = trimmed.split_whitespace().next_back() {
                let digits: String = pct_token.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(signal_pct) = digits.parse::<i32>() {
                    if let Some(bssid) = &current_bssid {
                        if let Some(entry) = strongest.get_mut(bssid) {
                            entry.signal_pct = signal_pct;
                        }
                    }
                }
            }
        }
    }

    finish(strongest)
}

fn scan_networkkit() -> Result<Vec<AccessPoint>, LocationError> {
    let networks = networkkit::Wifi::new()
        .scan(true)
        .map_err(|e| LocationError::ProviderFailed(format!("wifi scan failed: {e}")))?;

    let mut strongest: std::collections::HashMap<String, AccessPoint> =
        std::collections::HashMap::new();
    for net in networks {
        let Some(bssid) = net.bssid.as_deref().and_then(normalize_bssid) else {
            continue;
        };
        collect(&mut strongest, bssid, net.signal_pct);
    }

    Ok(finish(strongest))
}

pub fn scan_access_points() -> Result<Vec<AccessPoint>, LocationError> {
    if cfg!(target_os = "windows") {
        scan_netsh()
    } else {
        scan_networkkit()
    }
}

fn scan_netsh() -> Result<Vec<AccessPoint>, LocationError> {
    let output = std::process::Command::new("netsh")
        .args(["wlan", "show", "networks", "mode=bssid"])
        .output()
        .map_err(|e| LocationError::ProviderFailed(format!("netsh not available: {}", e)))?;

    if !output.status.success() {
        return Err(LocationError::ProviderFailed(format!(
            "netsh wifi scan failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(parse_netsh_output(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bssid_normalization() {
        assert_eq!(
            normalize_bssid("A0:F3:C1:3B:6F:90"),
            Some("a0:f3:c1:3b:6f:90".to_string())
        );
        assert_eq!(
            normalize_bssid("00-1c-42-1f-65-e9"),
            Some("00:1c:42:1f:65:e9".to_string())
        );
        assert_eq!(normalize_bssid("a0:f3:c1"), None);
        assert_eq!(normalize_bssid("zz:zz:zz:zz:zz:zz"), None);
        assert_eq!(normalize_bssid("hello"), None);
    }

    #[test]
    fn parses_netsh_output() {
        let sample = "\
Interface name : WLAN
There are 3 networks currently visible.

SSID 1 : HomeNet
    Network type            : Infrastructure
    Authentication          : WPA2-Personal
    Encryption              : CCMP
    BSSID 1                 : a0:f3:c1:3b:6f:90
         Signal             : 82%
         Radio type         : 802.11n
         Channel            : 6
    BSSID 2                 : a0:f3:c1:3b:6f:91
         Signal             : 40%
         Radio type         : 802.11n

SSID 2 : CafeGuest
    Authentication          : Open
    BSSID 1                 : 00-1C-42-1F-65-E9
         Signal             : 55%
";

        let aps = parse_netsh_output(sample);
        assert_eq!(aps.len(), 3);
        assert_eq!(aps[0].bssid, "a0:f3:c1:3b:6f:90");
        assert_eq!(aps[0].signal_pct, 82);
        assert_eq!(aps[1].bssid, "00:1c:42:1f:65:e9");
        assert_eq!(aps[1].signal_pct, 55);
        assert_eq!(aps[2].bssid, "a0:f3:c1:3b:6f:91");
        assert_eq!(aps[2].signal_pct, 40);
    }

    #[test]
    fn truncates_to_strongest() {
        let mut strongest = std::collections::HashMap::new();
        for i in 0..20 {
            collect(
                &mut strongest,
                format!("aa:bb:cc:dd:ee:{:02x}", i),
                (i as i32) * 5,
            );
        }

        let aps = finish(strongest);
        assert_eq!(aps.len(), MAX_ACCESS_POINTS);
        assert_eq!(aps[0].signal_pct, 95);
    }
}
