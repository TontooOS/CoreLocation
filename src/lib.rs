pub mod geocode;
pub mod providers;
pub mod types;

pub use types::{Coordinates, Location, LocationError, LocationSource};
pub use providers::LocationProvider;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CoreLocation {
    cache: Arc<Mutex<Option<(Location, Instant)>>>,
    cache_duration: Duration,
}

impl CoreLocation {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }

    pub fn get_location(&self) -> Result<Location, LocationError> {
        if let Ok(cache) = self.cache.lock() {
            if let Some((ref loc, time)) = *cache {
                if time.elapsed() < self.cache_duration {
                    return Ok(loc.clone());
                }
            }
        }

        let location = providers::get_best_location()?;

        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some((location.clone(), Instant::now()));
        }

        Ok(location)
    }

    pub fn get_location_from(&self, source: LocationSource) -> Result<Location, LocationError> {
        let providers = providers::get_all_providers();

        for provider in providers {
            if !provider.is_available() {
                continue;
            }

            let loc = match provider.get_location() {
                Ok(loc) => loc,
                Err(_) => continue,
            };
            if loc.source == source {
                return Ok(loc);
            }
        }

        Err(LocationError::ProviderFailed(format!(
            "Provider {:?} not available",
            source
        )))
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            *cache = None;
        }
    }

    pub fn available_providers(&self) -> Vec<String> {
        let providers = providers::get_all_providers();
        providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.name().to_string())
            .collect()
    }
}

impl Default for CoreLocation {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_location() -> Result<Location, LocationError> {
    CoreLocation::new().get_location()
}

pub fn get_location_from(source: LocationSource) -> Result<Location, LocationError> {
    CoreLocation::new().get_location_from(source)
}
