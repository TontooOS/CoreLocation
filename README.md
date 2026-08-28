# Tontoo CoreLocation

A Framework for Location, Users Location and Processing it.

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["CoreLocation"] }
```

Then at the crate root:

```rust
sdk::preinclude!();
use CoreLocation::{ /* ... */ };
```

## License

TCL v26.1
