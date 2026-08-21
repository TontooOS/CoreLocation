pub mod gps;
pub mod ip;
pub mod timezone;
pub mod wifi;

use crate::types::{Location, LocationError};

pub trait LocationProvider {
    fn name(&self) -> &str;
    fn get_location(&self) -> Result<Location, LocationError>;
    fn is_available(&self) -> bool;
}

pub fn get_all_providers() -> Vec<Box<dyn LocationProvider>> {
    vec![
        Box::new(gps::GpsProvider::new()),
        Box::new(wifi::WifiProvider::new()),
        Box::new(ip::IpProvider::new()),
        Box::new(timezone::TimezoneProvider::new()),
    ]
}

pub fn get_best_location() -> Result<Location, LocationError> {
    let providers = get_all_providers();
    let mut best: Option<Location> = None;

    for provider in &providers {
        if !provider.is_available() {
            continue;
        }

        match provider.get_location() {
            Ok(loc) => {
                if let Some(ref current) = best {
                    if loc.accuracy < current.accuracy {
                        best = Some(loc);
                    }
                } else {
                    best = Some(loc);
                }
            }
            Err(_) => continue,
        }
    }

    best.ok_or(LocationError::NoProvidersAvailable)
}
