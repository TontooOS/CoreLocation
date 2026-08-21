use crate::geocode;
use crate::providers::LocationProvider;
use crate::types::{Coordinates, Location, LocationError, LocationSource};
use std::collections::HashMap;

pub mod apple_wloc;
pub mod mylnikov;
pub mod scan;

const MIN_ACCURACY_M: f64 = 15.0;
const MAX_ACCURACY_M: f64 = 150.0;

pub struct WifiProvider;

impl WifiProvider {
    pub fn new() -> Self {
        Self
    }

    fn signal_weight(signal_pct: i32) -> f64 {
        let dbm = signal_pct as f64 / 2.0 - 100.0;
        10f64.powf(dbm / 10.0)
    }

    fn attach_address(&self, location: &mut Location) {
        if let Ok(addr) = geocode::reverse_geocode(location.coordinates) {
            *location = location.clone().with_address(&addr.city, &addr.country, &addr.region);
        }
    }

    fn locate_via_apple(&self, aps: &[scan::AccessPoint]) -> Result<Location, LocationError> {
        let bssids: Vec<String> = aps.iter().map(|ap| ap.bssid.clone()).collect();
        let results = apple_wloc::query(&bssids)?;

        if results.is_empty() {
            return Err(LocationError::ProviderFailed(
                "Apple WLOC returned no results".into(),
            ));
        }

        let mut signal_map: HashMap<&str, i32> = HashMap::new();
        for ap in aps {
            signal_map.insert(ap.bssid.as_str(), ap.signal_pct);
        }

        let mut matched: Vec<(f64, f64, f64)> = Vec::new();
        for r in &results {
            if let Some(signal_pct) = signal_map.get(r.bssid.as_str()) {
                matched.push((r.latitude, r.longitude, Self::signal_weight(*signal_pct)));
            }
        }

        let (lat, lon, accuracy) = if !matched.is_empty() {
            let weight_sum: f64 = matched.iter().map(|m| m.2).sum();
            let lat = matched.iter().map(|m| m.0 * m.2).sum::<f64>() / weight_sum;
            let lon = matched.iter().map(|m| m.1 * m.2).sum::<f64>() / weight_sum;

            let centroid = Coordinates::new(lat, lon);
            let spread = matched
                .iter()
                .map(|m| centroid.distance_to(&Coordinates::new(m.0, m.1)))
                .fold(0.0f64, f64::max);

            (lat, lon, (spread / 2.0).clamp(MIN_ACCURACY_M, MAX_ACCURACY_M))
        } else {
            let count = results.len() as f64;
            let lat = results.iter().map(|r| r.latitude).sum::<f64>() / count;
            let lon = results.iter().map(|r| r.longitude).sum::<f64>() / count;
            let accuracy = results
                .iter()
                .filter_map(|r| r.accuracy_m)
                .fold(f64::MAX, f64::min);
            let accuracy = if accuracy.is_finite() && accuracy > 0.0 {
                accuracy
            } else {
                50.0
            };
            (lat, lon, accuracy.max(MIN_ACCURACY_M))
        };

        let mut location = Location::new(Coordinates::new(lat, lon), accuracy, LocationSource::Wifi);
        self.attach_address(&mut location);
        Ok(location)
    }

    fn locate_via_mylnikov(&self, aps: &[scan::AccessPoint]) -> Result<Location, LocationError> {
        let pairs: Vec<(String, i32)> = aps
            .iter()
            .take(5)
            .map(|ap| (ap.bssid.clone(), ap.signal_pct))
            .collect();

        let (lat, lon, range) = mylnikov::query_multi(&pairs).or_else(|_| {
            let first = aps
                .first()
                .ok_or_else(|| LocationError::ProviderFailed("No access points".into()))?;
            mylnikov::query_single(&first.bssid)
        })?;

        let mut location = Location::new(
            Coordinates::new(lat, lon),
            range.max(50.0),
            LocationSource::Wifi,
        );
        self.attach_address(&mut location);
        Ok(location)
    }
}

impl Default for WifiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationProvider for WifiProvider {
    fn name(&self) -> &str {
        "WiFi Positioning"
    }

    fn is_available(&self) -> bool {
        scan::is_scanner_available()
    }

    fn get_location(&self) -> Result<Location, LocationError> {
        let aps = scan::scan_access_points()?;

        if aps.is_empty() {
            return Err(LocationError::ProviderFailed(
                "No WiFi access points found".into(),
            ));
        }

        self.locate_via_apple(&aps)
            .or_else(|_| self.locate_via_mylnikov(&aps))
    }
}
