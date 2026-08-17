use crate::types::{Location, LocationError};
use crate::providers::LocationProvider;

pub struct GpsProvider {
    port: Option<String>,
}

impl GpsProvider {
    pub fn new() -> Self {
        let ports = [
            "/dev/ttyUSB0",
            "/dev/ttyACM0",
            "/dev/ttyS0",
            "/dev/ttyAMA0",
            "/dev/gps0",
        ];

        let port = ports
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .map(|p| p.to_string());

        Self { port }
    }
}

impl LocationProvider for GpsProvider {
    fn name(&self) -> &str {
        "GPS"
    }

    fn is_available(&self) -> bool {
        self.port.is_some()
    }

    fn get_location(&self) -> Result<Location, LocationError> {
        Err(LocationError::ProviderFailed(
            "GPS serial reader not yet implemented".into(),
        ))
    }
}
