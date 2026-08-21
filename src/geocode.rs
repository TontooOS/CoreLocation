use crate::types::{Coordinates, LocationError};
use serde::Deserialize;

const USER_AGENT: &str = "TontooOS-CoreLocation/26.1 (TontooOS location framework)";

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub city: String,
    pub country: String,
    pub region: String,
}

#[derive(Deserialize)]
struct NominatimResponse {
    address: Option<NominatimAddress>,
}

#[derive(Deserialize)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

pub fn reverse_geocode(coords: Coordinates) -> Result<AddressInfo, LocationError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| LocationError::NetworkError(e.to_string()))?;

    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={}&lon={}&zoom=10",
        coords.latitude, coords.longitude
    );

    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .map_err(|e| LocationError::NetworkError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(LocationError::ProviderFailed(format!(
            "Nominatim returned status {}",
            resp.status()
        )));
    }

    let json: NominatimResponse = resp
        .json()
        .map_err(|e| LocationError::ParseError(e.to_string()))?;

    let addr = json
        .address
        .ok_or_else(|| LocationError::ParseError("Nominatim response missing address".into()))?;

    let city = addr
        .city
        .or(addr.town)
        .or(addr.village)
        .or(addr.municipality)
        .ok_or_else(|| LocationError::ParseError("Nominatim response missing city".into()))?;

    Ok(AddressInfo {
        city,
        country: addr.country.unwrap_or_default(),
        region: addr.state.unwrap_or_default(),
    })
}
