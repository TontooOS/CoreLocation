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
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={}&lon={}&zoom=10",
        coords.latitude, coords.longitude
    );

    let resp = networkkit::http::HttpRequest::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .header("User-Agent", USER_AGENT)
        .send()?;

    if !resp.is_success() {
        return Err(LocationError::ProviderFailed(format!(
            "Nominatim returned status {}",
            resp.status
        )));
    }

    let json: NominatimResponse = resp.json()?;

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
