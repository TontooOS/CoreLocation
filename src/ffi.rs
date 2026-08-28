//! C FFI exports for CoreLocation.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde_json::json;

use crate::types::LocationSource;

unsafe fn read_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(str::to_owned)
}

fn set_error(error_out: *mut *mut c_char, message: &str) {
    if error_out.is_null() {
        return;
    }
    if let Ok(c) = CString::new(message.to_owned()) {
        unsafe { *error_out = c.into_raw() };
    }
}

fn json_ptr(value: &serde_json::Value) -> *mut c_char {
    CString::new(value.to_string())
        .unwrap_or_default()
        .into_raw()
}

/// The framework version as a static C string.
#[no_mangle]
pub extern "C" fn tontoo_corelocation_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Resolve the current location. Returns a JSON object or null.
///
/// JSON keys: `latitude`, `longitude`, `accuracy`, `source`, `city`,
/// `country`, `region`. Blocking - call from a worker thread.
///
/// # Safety
///
/// `error_out`, when not null, must point to a writable `char*`.
#[no_mangle]
pub unsafe extern "C" fn tontoo_corelocation_get_location(
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match crate::get_location() {
        Ok(location) => json_ptr(&location_json(&location)),
        Err(_) => {
            set_error(error_out, "location unavailable");
            std::ptr::null_mut()
        }
    }
}

/// Resolve the current location from a specific source.
///
/// `source`: 0 GPS, 1 WiFi, 2 IP, 3 timezone, 4 manual. Returns a JSON
/// object or null. Blocking - call from a worker thread.
///
/// # Safety
///
/// `error_out`, when not null, must point to a writable `char*`.
#[no_mangle]
pub unsafe extern "C" fn tontoo_corelocation_get_location_from(
    source: i32,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    let source = match source {
        0 => LocationSource::Gps,
        1 => LocationSource::Wifi,
        2 => LocationSource::Ip,
        3 => LocationSource::Timezone,
        4 => LocationSource::Manual,
        _ => {
            set_error(error_out, "invalid source");
            return std::ptr::null_mut();
        }
    };
    match crate::get_location_from(source) {
        Ok(location) => json_ptr(&location_json(&location)),
        Err(_) => {
            set_error(error_out, "location unavailable");
            std::ptr::null_mut()
        }
    }
}

fn location_json(location: &crate::types::Location) -> serde_json::Value {
    json!({
        "latitude": location.coordinates.latitude,
        "longitude": location.coordinates.longitude,
        "accuracy": location.accuracy,
        "source": location.source.to_string(),
        "city": location.city,
        "country": location.country,
        "region": location.region,
    })
}

/// Free a string returned by this library.
///
/// # Safety
///
/// `s` must be a pointer returned by this API or null.
#[no_mangle]
pub unsafe extern "C" fn tontoo_corelocation_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
