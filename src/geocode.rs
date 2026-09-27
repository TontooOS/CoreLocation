use crate::types::{Coordinates, LocationError};
use foundation::serialization::JsonDocument;

const USER_AGENT: &str = "TontooOS-CoreLocation/26.1 (TontooOS location framework)";

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub city: String,
    pub country: String,
    pub region: String,
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

    let body = resp.text().map_err(|e| LocationError::ParseError(e.to_string()))?;
    let doc = JsonDocument::parse(&body).map_err(|e| LocationError::ParseError(e.to_string()))?;
    let addr = doc
        .nested("address")
        .map_err(|e| LocationError::ParseError(e.to_string()))?
        .ok_or_else(|| LocationError::ParseError("Nominatim response missing address".into()))?;

    let field = |name: &str| {
        addr.str_field(name)
            .map_err(|e| LocationError::ParseError(e.to_string()))
    };
    let city = field("city")?
        .or(field("town")?)
        .or(field("village")?)
        .or(field("municipality")?)
        .ok_or_else(|| LocationError::ParseError("Nominatim response missing city".into()))?;

    Ok(AddressInfo {
        city,
        country: field("country")?.unwrap_or_default(),
        region: field("state")?.unwrap_or_default(),
    })
}
