use crate::types::LocationError;
use serde::Deserialize;
use std::time::Duration;

const API_URL: &str = "https://api.mylnikov.org/geolocation/wifi";

#[derive(Deserialize)]
struct MylnikovResponse {
    result: i32,
    data: Option<MylnikovData>,
}

#[derive(Deserialize)]
struct MylnikovData {
    lat: Option<f64>,
    lon: Option<f64>,
    range: Option<f64>,
}

const BASE64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(BASE64_ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(BASE64_ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            BASE64_ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64_ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn signal_pct_to_dbm(signal_pct: i32) -> i32 {
    signal_pct / 2 - 100
}

fn request(url: &str) -> Result<(f64, f64, f64), LocationError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| LocationError::NetworkError(e.to_string()))?;

    let resp = client
        .get(url)
        .send()
        .map_err(|e| LocationError::NetworkError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(LocationError::ProviderFailed(format!(
            "Mylnikov API returned status {}",
            resp.status()
        )));
    }

    let json: MylnikovResponse = resp
        .json()
        .map_err(|e| LocationError::ParseError(e.to_string()))?;

    if json.result != 200 {
        return Err(LocationError::ProviderFailed(format!(
            "Mylnikov API result {}",
            json.result
        )));
    }

    let data = json
        .data
        .ok_or_else(|| LocationError::ParseError("Mylnikov response missing data".into()))?;

    let lat = data
        .lat
        .ok_or_else(|| LocationError::ParseError("Mylnikov response missing lat".into()))?;
    let lon = data
        .lon
        .ok_or_else(|| LocationError::ParseError("Mylnikov response missing lon".into()))?;

    Ok((lat, lon, data.range.unwrap_or(150.0)))
}

pub fn query_multi(aps: &[(String, i32)]) -> Result<(f64, f64, f64), LocationError> {
    let pairs: Vec<String> = aps
        .iter()
        .map(|(bssid, signal_pct)| format!("{},{}", bssid.to_lowercase(), signal_pct_to_dbm(*signal_pct)))
        .collect();

    let search = base64_encode(pairs.join(";").as_bytes());
    let url = format!("{}?v=1.1&data=open&search={}", API_URL, search);
    request(&url)
}

pub fn query_single(bssid: &str) -> Result<(f64, f64, f64), LocationError> {
    let url = format!("{}?v=1.1&data=open&bssid={}", API_URL, bssid);
    request(&url)
}
