# CoreLocation – Wiki

CoreLocation is the location framework for TontooOS. It combines GPS, WiFi, IP and
timezone based positioning and always returns the most accurate fix available,
without requiring any API keys.

- Repository: https://github.com/TontooOS/Libs
- License: TCL
- Version: 27.0.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Location | [Location.md](Location.md) | `CoreLocation` client, caching, blocking and async API |
| Providers | [Providers.md](Providers.md) | Provider trait, provider priority, GPS/IP/Timezone providers |
| WiFi Positioning | [WifiPositioning.md](WifiPositioning.md) | Access point scanning, Apple WLOC, Mylnikov fallback, reverse geocoding |
| Localization | [Localization.md](Localization.md) | Error message localization via `lang/en_us.json` and `lang/de_de.json` |

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

- 2026-09-26: HTTP transport moved to NetworkKit (`networkkit::http`) in
  the IP, geocode, Apple WLOC and Mylnikov providers; the Linux WiFi scan
  now uses `networkkit::Wifi` instead of a private `nmcli` parser. The
  `reqwest` dependency is removed.
- 2026-08-21: Added async API (`get_location_async`), error message localization
  (`src/lang.rs`, `lang/`) and 19 offline unit tests for the WiFi protocol,
  parsers and localization.
- 2026-08-21: Added a Windows `netsh` scanner so WiFi positioning works on
  Windows too; verified live with a ±32m fix.
- 2026-08-21: Added the WiFi positioning provider (Apple WLOC + Mylnikov +
  Nominatim), new `LocationSource::Wifi` variant, public `geocode` module and
  fixed error propagation in `get_location_from`.
- 2026-08-21: Initial wiki with Location, Providers and WiFi Positioning pages.
