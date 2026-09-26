# WiFi Positioning

The WiFi provider locates a machine without GPS by scanning nearby access
points and resolving their positions against public WiFi databases. No API keys
are required. It works on Linux with NetworkManager (TontooOS ships it by
default) and on Windows via `netsh`, without root in both cases.

## Resolution Chain

`WifiProvider::get_location()` runs the following steps:

1. Scan access points (`scan::scan_access_points`): NetworkKit WiFi
   (`networkkit::Wifi::scan`) on Linux,
   `netsh wlan show networks mode=bssid` on Windows.
2. Query Apple's WLOC service with all BSSIDs (`apple_wloc::query`) and compute
   an RSSI-weighted centroid.
3. If Apple returns nothing, fall back to the Mylnikov open database
   (`mylnikov::query_multi`, then `mylnikov::query_single`).
4. Reverse geocode the final coordinates via OpenStreetMap Nominatim
   (`geocode::reverse_geocode`) to fill city, country and region.

Accuracy of the final fix is typically 15–150 meters.

## WifiProvider

```rust
pub struct WifiProvider;

impl WifiProvider {
    pub fn new() -> Self;
}
```

Implements `LocationProvider`:

| Method | Behavior |
|---|---|
| `name()` | Returns `"WiFi Positioning"` |
| `is_available()` | True when a scanner exists (NetworkKit WiFi on Linux, `netsh` present on Windows) |
| `get_location()` | Runs the resolution chain; returns `Err` when no APs are found or both databases fail |

## scan

```rust
pub struct AccessPoint {
    pub bssid: String,
    pub signal_pct: i32,
}

pub fn is_scanner_available() -> bool
pub fn scan_access_points() -> Result<Vec<AccessPoint>, LocationError>
```

- On Linux it scans through NetworkKit (`networkkit::Wifi::scan` with an
  active rescan); BSSIDs without an address are skipped.
- On Windows it runs `netsh wlan show networks mode=bssid`. Parsing is
  language independent: BSSID lines are detected by MAC address pattern and
  signal lines by percentage value.
- Duplicate BSSIDs are reduced to the strongest signal; results are sorted by
  signal strength and truncated to the 12 strongest APs.
- Returns `Err(LocationError::ProviderFailed)` when no scanner is available or
  the scan fails.

## apple_wloc

```rust
pub struct WifiApResult {
    pub bssid: String,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: Option<f64>,
}

pub fn query(bssids: &[String]) -> Result<Vec<WifiApResult>, LocationError>
```

Client for Apple's undocumented but keyless WPS endpoint
`https://gs-loc.apple.com/clls/wloc`. The request wraps a protobuf
`AppleWLoc` message in the ARPC framing used by iOS/macOS `locationd`; the
response protobuf lists up to 400 nearby access points with coordinates scaled
by 1e-8.

- Entries with invalid coordinates (-180/-180) are dropped.
- Returns `Err(LocationError::NetworkError)` on request failure and
  `Err(LocationError::ParseError)` on malformed responses.

> **Note:** This endpoint is undocumented and may change without notice.

## mylnikov

```rust
pub fn query_single(bssid: &str) -> Result<(f64, f64, f64), LocationError>
pub fn query_multi(aps: &[(String, i32)]) -> Result<(f64, f64, f64), LocationError>
```

Client for the free Mylnikov geo-location database
(`https://api.mylnikov.org/geolocation/wifi`, open data, no key).

- `query_single` resolves one BSSID; returns `(lat, lon, range_m)`.
- `query_multi` base64-encodes up to five `bssid,dbm` pairs into the `search`
  parameter; signal percentages are converted to dBm via `pct / 2 - 100`.
- Returns `Err(LocationError::ProviderFailed)` when the API answers with a
  non-200 result code or HTTP error.

## geocode

```rust
pub struct AddressInfo {
    pub city: String,
    pub country: String,
    pub region: String,
}

pub fn reverse_geocode(coords: Coordinates) -> Result<AddressInfo, LocationError>
```

Reverse geocoding via OpenStreetMap Nominatim (`zoom=10`). The module is public
and can be used standalone for GPS or IP coordinates as well.

- City falls back through `city` -> `town` -> `village` -> `municipality`.
- Requests identify as `TontooOS-CoreLocation/26.1`; respect the Nominatim
  1 request/second limit.
- Returns `Err(LocationError::ParseError)` when no address or city is present.

## Centroid Calculation

When Apple returns positions for scanned BSSIDs, each point is weighted by
linear signal power `w = 10^(dbm / 10)` and combined into a weighted centroid.
Accuracy is half the maximum distance from the centroid to any used point,
clamped to 15–150 meters. When none of the scanned BSSIDs match, the provider
averages all returned neighbor points instead and uses the smallest reported
accuracy value.

## Usage / Example

```rust
use corelocation::providers::wifi::WifiProvider;
use corelocation::providers::LocationProvider;

let provider = WifiProvider::new();
if provider.is_available() {
    let loc = provider.get_location().unwrap();
    println!("{}", loc);
    // 52.520000, 13.405000 (±38m), Berlin, Germany
}
```

Test the backends directly without WiFi hardware:

```bash
cargo run --example test_wifi
```

## Cross References

- [Providers.md](Providers.md) – where the WiFi provider sits in the chain
- [Location.md](Location.md) – types returned by the provider
