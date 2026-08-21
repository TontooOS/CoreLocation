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
        is_nmcli_available()
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

fn parse_nmcli_output(stdout: &str) -> Vec<AccessPoint> {
    let mut strongest: std::collections::HashMap<String, AccessPoint> =
        std::collections::HashMap::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            continue;
        }

        let bssid = parts[..6].join(":");
        if bssid.len() != 17 || !bssid.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
            continue;
        }

        let signal_pct: i32 = match parts[6].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        collect(&mut strongest, bssid, signal_pct);
    }

    finish(strongest)
}

fn is_nmcli_available() -> bool {
    std::process::Command::new("nmcli")
        .args(["-t", "-f", "RUNNING", "general"])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "running")
        .unwrap_or(false)
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

fn scan_nmcli() -> Result<Vec<AccessPoint>, LocationError> {
    let output = std::process::Command::new("nmcli")
        .args([
            "-t",
            "--escape",
            "yes",
            "-f",
            "BSSID,SIGNAL",
            "dev",
            "wifi",
            "list",
            "--rescan",
            "yes",
        ])
        .output()
        .map_err(|e| LocationError::ProviderFailed(format!("nmcli not available: {}", e)))?;

    if !output.status.success() {
        return Err(LocationError::ProviderFailed(format!(
            "nmcli wifi scan failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(parse_nmcli_output(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

pub fn scan_access_points() -> Result<Vec<AccessPoint>, LocationError> {
    if cfg!(target_os = "windows") {
        scan_netsh()
    } else {
        scan_nmcli()
    }
}
