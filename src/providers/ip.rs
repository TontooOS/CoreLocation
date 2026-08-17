use crate::types::{Coordinates, Location, LocationError, LocationSource};
use crate::providers::LocationProvider;
use serde::Deserialize;

#[derive(Deserialize)]
struct IpResponse {
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    country: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    status: Option<String>,
}

pub struct IpProvider;

impl IpProvider {
    pub fn new() -> Self {
        Self
    }

    fn try_api(&self, url: &str) -> Result<Location, LocationError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| LocationError::NetworkError(e.to_string()))?;

        let resp = client
            .get(url)
            .send()
            .map_err(|e| LocationError::NetworkError(e.to_string()))?;

        let ip: IpResponse = resp
            .json()
            .map_err(|e| LocationError::ParseError(e.to_string()))?;

        if ip.status.as_deref() == Some("fail") || ip.lat.is_none() || ip.lon.is_none() {
            return Err(LocationError::ProviderFailed(
                "IP API returned no data".into(),
            ));
        }

        let coords = Coordinates::new(ip.lat.unwrap(), ip.lon.unwrap());
        let mut loc = Location::new(coords, 5000.0, LocationSource::Ip);

        if let (Some(city), Some(country)) = (&ip.city, &ip.country) {
            loc = loc.with_address(
                city,
                country,
                ip.region_name.as_deref().unwrap_or(""),
            );
        }

        Ok(loc)
    }
}

impl LocationProvider for IpProvider {
    fn name(&self) -> &str {
        "IP Geolocation"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn get_location(&self) -> Result<Location, LocationError> {
        let apis = [
            "http://ip-api.com/json/",
            "https://ipinfo.io/json",
        ];

        for api in &apis {
            if let Ok(loc) = self.try_api(api) {
                return Ok(loc);
            }
        }

        Err(LocationError::ProviderFailed(
            "All IP APIs failed".into(),
        ))
    }
}
