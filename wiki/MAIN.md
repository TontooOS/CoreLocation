# CoreLocation – Wiki

CoreLocation is the location framework for TontooOS. It combines GPS, WiFi, IP and
timezone based positioning and always returns the most accurate fix available,
without requiring any API keys.

- Repository: https://github.com/TontooOS/Libs
- License: TCL
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Location | [Location.md](Location.md) | `CoreLocation` client, caching, types and errors |
| Providers | [Providers.md](Providers.md) | Provider trait, provider priority, GPS/IP/Timezone providers |
| WiFi Positioning | [WifiPositioning.md](WifiPositioning.md) | Access point scanning, Apple WLOC, Mylnikov fallback, reverse geocoding |

## Quick Start

```rust
use corelocation::get_location;

fn main() {
    let loc = get_location().unwrap();
    println!("{}", loc);
    // 52.520000, 13.405000 (±42m), Berlin, Germany
}
```

See [Location.md](Location.md) for details.

## Changelog

- 2026-08-21: Added a Windows `netsh` scanner so WiFi positioning works on
  Windows too; verified live with a ±32m fix.
- 2026-08-21: Added the WiFi positioning provider (Apple WLOC + Mylnikov +
  Nominatim), new `LocationSource::Wifi` variant, public `geocode` module and
  fixed error propagation in `get_location_from`.
- 2026-08-21: Initial wiki with Location, Providers and WiFi Positioning pages.
