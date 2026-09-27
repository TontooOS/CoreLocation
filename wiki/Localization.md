# Localization

CoreLocation localizes its error messages. Per TontooOS convention there are
exactly two language files, `lang/en_us.json` and `lang/de_de.json`. Both are
compiled into the library with `include_str!`, so no file lookup happens at
runtime and the module works everywhere.

## Language Detection

```rust
pub fn current_locale() -> &'static str
```

Reads the system locale from the environment in this order:

1. `LC_ALL`
2. `LC_MESSAGES`
3. `LANG`

A value starting with `de` (case insensitive) selects `de_de`; everything else
falls back to `en_us`.

## Translation Lookup

```rust
pub fn t(key: &str) -> String
pub fn t_fmt(key: &str, arg: &str) -> String
```

| Function | Behavior |
|---|---|
| `t(key)` | Returns the translation for `key`; unknown keys return the key itself |
| `t_fmt(key, arg)` | Like `t`, but replaces the `{}` placeholder with `arg` |

Messages are parsed once on first use (`OnceLock`) and cached for the process
lifetime. Parsing uses Foundation's std-only `JSONSerialization::parse_flat_string_map`,
so this crate has no `serde` dependency.

## Message Keys

| Key | Used by |
|---|---|
| `no_providers_available` | `LocationError::NoProvidersAvailable` |
| `provider_failed` | `LocationError::ProviderFailed` |
| `network_error` | `LocationError::NetworkError` |
| `parse_error` | `LocationError::ParseError` |
| `timeout` | `LocationError::Timeout` |
| `permission_denied` | `LocationError::PermissionDenied` |

`Display` of `LocationError` uses these translations automatically; callers do
not need to touch the `lang` module.

## File Format

```json
{
  "no_providers_available": "Keine Standort-Dienste verfuegbar",
  "provider_failed": "Anbieter fehlgeschlagen: {}"
}
```

Rules: only `en_us.json` and `de_de.json` exist; every variant of
`LocationError` has a key; `{}` marks the single placeholder slot.

## Usage / Example

```rust
use corelocation::lang;

println!("{}", lang::current_locale());
println!("{}", lang::t_fmt("provider_failed", "gpsd not reachable"));
```

## Cross References

- [Location.md](Location.md) – where these messages surface (`LocationError`)
