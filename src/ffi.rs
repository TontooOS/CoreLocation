//! C FFI exports for CoreLocation.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::types::LocationSource;
use foundation::serialization::JsonObject;

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

fn json_ptr(json: &str) -> *mut c_char {
    CString::new(json).unwrap_or_default().into_raw()
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

fn location_json(location: &crate::types::Location) -> String {
    let mut obj = JsonObject::new();
    obj.field_f64("latitude", location.coordinates.latitude)
        .and_then(|o| o.field_f64("longitude", location.coordinates.longitude))
        .and_then(|o| o.field_f64("accuracy", location.accuracy))
        .ok();
    obj.field_str("source", &location.source.to_string());
    obj.field_opt_str("city", location.city.as_deref());
    obj.field_opt_str("country", location.country.as_deref());
    obj.field_opt_str("region", location.region.as_deref());
    obj.build(false).unwrap_or_else(|_| "{}".to_string())
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
