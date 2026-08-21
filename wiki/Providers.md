# Providers

Every positioning backend implements the `LocationProvider` trait. Providers are
registered in a fixed order and queried by the `CoreLocation` client.

## Trait

```rust
pub trait LocationProvider {
    fn name(&self) -> &str;
    fn get_location(&self) -> Result<Location, LocationError>;
    fn is_available(&self) -> bool;
}
```

| Method | Description |
|---|---|
| `name` | Human readable provider name shown by `available_providers` |
| `get_location` | Returns a fix or an error; must never block indefinitely |
| `is_available` | Cheap check whether the backend can run on this system |

## Registration Order

```rust
pub fn get_all_providers() -> Vec<Box<dyn LocationProvider>> {
    vec![
        Box::new(gps::GpsProvider::new()),
        Box::new(wifi::WifiProvider::new()),
        Box::new(ip::IpProvider::new()),
        Box::new(timezone::TimezoneProvider::new()),
    ]
}
```

## Provider Priority

`get_best_location()` queries **all** available providers sequentially and keeps
the result with the lowest accuracy value. Typical outcome per environment:

| Environment | Winning source | Typical accuracy |
|---|---|---|
| GPS hardware present | `Gps` | 5–10m |
| NetworkManager + WiFi | `Wifi` | 15–150m |
| Online, no WiFi scan | `Ip` | ~5km |
| Offline | `Timezone` | ~50km |

> **Note:** All providers are queried even when an early fix exists. A failing
> provider never aborts the chain; its error is skipped.

## GpsProvider

Serial port based GPS receiver support. Scans `/dev/ttyUSB0`, `/dev/ttyACM0`,
`/dev/ttyS0`, `/dev/ttyAMA0` and `/dev/gps0` for existence.

- `is_available()` returns true when one of the ports exists.
- `get_location()` currently always returns
  `Err(LocationError::ProviderFailed)`; the serial reader is not implemented yet.

## IpProvider

Queries public IP geolocation APIs in order:

1. `http://ip-api.com/json/`
2. `https://ipinfo.io/json`

Each request has a 5 second timeout. The first successful response wins.
Accuracy is hardcoded to 5000 meters; city, country and region are taken from
the API response.

- Returns `Err(LocationError::ProviderFailed)` when all APIs fail or return no
  coordinates.

## TimezoneProvider

Reads the system timezone and maps it to representative city coordinates of a
built-in table (about 60 timezones).

- On Linux it reads `/etc/timezone`, falling back to `timedatectl`.
- On Windows it converts Windows timezone IDs to IANA names via a lookup table.
- Accuracy is hardcoded to 50000 meters.
- Returns `Err(LocationError::ProviderFailed)` when the timezone is missing or
  not in the table.

## Usage / Example

```rust
use corelocation::providers::{LocationProvider, get_all_providers};

for provider in get_all_providers() {
    if provider.is_available() {
        println!("{}: {:?}", provider.name(), provider.get_location());
    }
}
```

## Cross References

- [Location.md](Location.md) – how the client picks the best result
- [WifiPositioning.md](WifiPositioning.md) – internals of the WiFi provider
