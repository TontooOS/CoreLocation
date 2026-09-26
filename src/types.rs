use crate::lang;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
        }
    }

    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        let r = 6371000.0;
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();

        let a = (dlat / 2.0).sin() * (dlat / 2.0).sin()
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin() * (dlon / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}, {:.6}", self.latitude, self.longitude)
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub coordinates: Coordinates,
    pub accuracy: f64,
    pub source: LocationSource,
    pub city: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
}

impl Location {
    pub fn new(coords: Coordinates, accuracy: f64, source: LocationSource) -> Self {
        Self {
            coordinates: coords,
            accuracy,
            source,
            city: None,
            country: None,
            region: None,
        }
    }

    pub fn with_address(mut self, city: &str, country: &str, region: &str) -> Self {
        self.city = Some(city.to_string());
        self.country = Some(country.to_string());
        self.region = Some(region.to_string());
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = match (&self.city, &self.country) {
            (Some(city), Some(country)) => format!(", {}, {}", city, country),
            (Some(city), None) => format!(", {}", city),
            _ => String::new(),
        };
        write!(
            f,
            "{} (±{:.0}m){}",
            self.coordinates, self.accuracy, addr
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationSource {
    Gps,
    Wifi,
    Ip,
    Timezone,
    Manual,
}

impl fmt::Display for LocationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationSource::Gps => write!(f, "GPS"),
            LocationSource::Wifi => write!(f, "WiFi"),
            LocationSource::Ip => write!(f, "IP"),
            LocationSource::Timezone => write!(f, "Timezone"),
            LocationSource::Manual => write!(f, "Manual"),
        }
    }
}

#[derive(Debug)]
pub enum LocationError {
    NoProvidersAvailable,
    ProviderFailed(String),
    NetworkError(String),
    ParseError(String),
    Timeout,
    PermissionDenied,
}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationError::NoProvidersAvailable => write!(f, "{}", lang::t("no_providers_available")),
            LocationError::ProviderFailed(p) => write!(f, "{}", lang::t_fmt("provider_failed", p)),
            LocationError::NetworkError(e) => write!(f, "{}", lang::t_fmt("network_error", e)),
            LocationError::ParseError(e) => write!(f, "{}", lang::t_fmt("parse_error", e)),
            LocationError::Timeout => write!(f, "{}", lang::t("timeout")),
            LocationError::PermissionDenied => write!(f, "{}", lang::t("permission_denied")),
        }
    }
}

impl std::error::Error for LocationError {}

impl From<networkkit::types::NetworkError> for LocationError {
    fn from(err: networkkit::types::NetworkError) -> Self {
        use networkkit::types::NetworkError as NetErr;
        match err {
            NetErr::Timeout => LocationError::Timeout,
            NetErr::ParseError(msg) => LocationError::ParseError(msg),
            NetErr::PermissionDenied => LocationError::PermissionDenied,
            other => LocationError::NetworkError(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_berlin_munich() {
        let berlin = Coordinates::new(52.5200, 13.4050);
        let munich = Coordinates::new(48.1351, 11.5820);
        let dist = berlin.distance_to(&munich);
        assert!((dist - 504_000.0).abs() < 5_000.0, "got {}m", dist);
    }

    #[test]
    fn display_formats() {
        let coords = Coordinates::new(52.52, 13.405);
        assert_eq!(coords.to_string(), "52.520000, 13.405000");

        let loc = Location::new(coords, 42.0, LocationSource::Wifi)
            .with_address("Idstein", "Deutschland", "Hessen");
        assert_eq!(
            loc.to_string(),
            "52.520000, 13.405000 (±42m), Idstein, Deutschland"
        );
    }

    #[test]
    fn error_messages_localized() {
        assert_eq!(
            LocationError::NoProvidersAvailable.to_string(),
            lang::t("no_providers_available")
        );
        assert_eq!(
            LocationError::Timeout.to_string(),
            lang::t("timeout")
        );
    }
}
