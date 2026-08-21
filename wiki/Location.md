# Location

The `CoreLocation` client is the entry point of the framework. It queries all
available providers, caches results and returns the most accurate location.

## Constructors

### CoreLocation::new

```rust
pub fn new() -> Self
```

Creates a client with a cache duration of 300 seconds.

### CoreLocation::with_cache_duration

```rust
pub fn with_cache_duration(mut self, duration: Duration) -> Self
```

Builder method that overrides the cache duration. Cached locations are returned
without querying any provider until the duration expires.

## Methods

### get_location

```rust
pub fn get_location(&self) -> Result<Location, LocationError>
```

Returns the cached location if it is still valid. Otherwise it calls
`providers::get_best_location()`, which queries every available provider and
keeps the result with the lowest accuracy value (in meters). The result is
cached.

- Returns `Err(LocationError::NoProvidersAvailable)` when no provider returns a
  fix.

### get_location_from

```rust
pub fn get_location_from(&self, source: LocationSource) -> Result<Location, LocationError>
```

Iterates all available providers in registration order and returns the first
result whose source matches. Providers that fail are skipped silently.

- Returns `Err(LocationError::ProviderFailed)` when no provider with the
  requested source succeeds.

### clear_cache

```rust
pub fn clear_cache(&self)
```

Drops the cached location so the next call to `get_location` queries providers
again.

### available_providers

```rust
pub fn available_providers(&self) -> Vec<String>
```

Returns the names of all providers whose `is_available()` currently returns
true.

## Free Functions

```rust
pub fn get_location() -> Result<Location, LocationError>
pub fn get_location_from(source: LocationSource) -> Result<Location, LocationError>
```

Convenience wrappers that create a fresh `CoreLocation` per call. No caching
across calls.

## Types

### Coordinates

```rust
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}
```

| Method | Description |
|---|---|
| `Coordinates::new(lat, lon)` | Creates coordinates from f64 values |
| `coords.distance_to(&other)` | Great-circle distance in meters (Haversine) |

`Display` renders as `"52.520000, 13.405000"`.

### Location

```rust
pub struct Location {
    pub coordinates: Coordinates,
    pub accuracy: f64,
    pub source: LocationSource,
    pub city: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
}
```

| Method | Description |
|---|---|
| `Location::new(coords, accuracy, source)` | Creates a location without address data |
| `loc.with_address(city, country, region)` | Fills city, country and region |

`accuracy` is the estimated error radius in meters; lower is better.
`Display` renders as `"52.520000, 13.405000 (±42m), Berlin, Germany"`.

### LocationSource

```rust
pub enum LocationSource {
    Gps,
    Wifi,
    Ip,
    Timezone,
    Manual,
}
```

`Display` renders `"GPS"`, `"WiFi"`, `"IP"`, `"Timezone"`, `"Manual"`.

### LocationError

| Variant | Meaning |
|---|---|
| `NoProvidersAvailable` | No provider returned a fix |
| `ProviderFailed(String)` | A provider failed; message contains details |
| `NetworkError(String)` | HTTP request failed |
| `ParseError(String)` | Response could not be parsed |
| `Timeout` | Request timed out |
| `PermissionDenied` | Access to a resource was denied |

Implements `std::error::Error`.

## Usage / Example

```rust
use corelocation::{CoreLocation, LocationSource};
use std::time::Duration;

let client = CoreLocation::new().with_cache_duration(Duration::from_secs(60));

let best = client.get_location().unwrap();
println!("Best: {}", best);

let wifi = client.get_location_from(LocationSource::Wifi);
match wifi {
    Ok(loc) => println!("WiFi fix: {}", loc),
    Err(e) => println!("WiFi unavailable: {}", e),
}
```

## Cross References

- [Providers.md](Providers.md) – how providers are selected and prioritized
- [WifiPositioning.md](WifiPositioning.md) – internals of the WiFi provider
