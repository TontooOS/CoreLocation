# Tontoo CoreLocation

A Framework for Location, Users Location and Processing it.

CoreLocation combines multiple positioning sources and always returns the most accurate fix available — no API keys required anywhere.

## Wiki

Documentation lives in [wiki/MAIN.md](wiki/MAIN.md).

## Positioning Sources

| Source | Accuracy | How it works |
|---|---|---|
| **GPS** | ~5–10m | `gpsd` / GPS hardware (see `providers/gps.rs`) |
| **WiFi** | ~15–150m | Scans nearby access points via NetworkManager (`nmcli`), then resolves position with Apple's WLOC service (`gs-loc.apple.com/clls/wloc`, RSSI-weighted centroid over returned APs). Falls back to the free Mylnikov WiFi database (`api.mylnikov.org`). Addresses are resolved via OpenStreetMap Nominatim reverse geocoding. |
| **IP** | ~5km | ip-api.com / ipinfo.io |
| **Timezone** | ~50km | System timezone → city coordinates |

Priority: the best (lowest accuracy value) result across all available providers wins. Typical order: GPS > WiFi > IP > Timezone.

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
corelocation = { path = "/Library/System/corelocation.library" }
```

## Usage

```rust
use corelocation::{get_location, get_location_from, LocationSource};

// Best available location (tries all providers, keeps the most accurate)
let loc = get_location()?;
println!("{}", loc); // 52.520000, 13.405000 (±42m), Berlin, Germany

// Force a specific source
let wifi_loc = get_location_from(LocationSource::Wifi)?;
```

### Manual provider usage

```rust
use corelocation::providers::wifi::WifiProvider;
use corelocation::providers::LocationProvider;

let provider = WifiProvider::new();
if provider.is_available() {
    let loc = provider.get_location()?;
}
```

Submodules:

- `corelocation::providers::wifi::scan` — nmcli access point scanning
- `corelocation::providers::wifi::apple_wloc` — Apple WLOC protobuf client
- `corelocation::providers::wifi::mylnikov` — Mylnikov API client
- `corelocation::geocode` — Nominatim reverse geocoding (`Coordinates -> AddressInfo`)

## Notes

- WiFi positioning uses `nmcli` (Linux/NetworkManager) or `netsh` (Windows) and works without root.
- All remote services used are free and keyless: Apple WLOC (undocumented but public), Mylnikov API (open data), OSM Nominatim (1 req/s limit).
- Results are cached for 300s by default (`CoreLocation::with_cache_duration`).

## Testing

```bash
cargo run --example test_location   # full provider chain
cargo run --example test_wifi       # WLOC + Mylnikov + Nominatim backends directly
```

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## License

TCL v26.1
