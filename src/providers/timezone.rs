use crate::types::{Coordinates, Location, LocationError, LocationSource};
use crate::providers::LocationProvider;

pub struct TimezoneProvider;

impl TimezoneProvider {
    pub fn new() -> Self {
        Self
    }

    fn get_system_timezone(&self) -> Option<String> {
        if cfg!(target_os = "linux") {
            std::fs::read_to_string("/etc/timezone")
                .ok()
                .map(|s| s.trim().to_string())
                .or_else(|| {
                    std::process::Command::new("timedatectl")
                        .arg("show")
                        .arg("--property=Timezone")
                        .arg("--value")
                        .output()
                        .ok()
                        .and_then(|o| {
                            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                            if s.is_empty() { None } else { Some(s) }
                        })
                })
        } else if cfg!(target_os = "windows") {
            std::process::Command::new("powershell")
                .args(["-Command", "[System.TimeZoneInfo]::Local.Id"])
                .output()
                .ok()
                .and_then(|o| {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                })
                .map(|tz| self.windows_to_iana(&tz))
        } else {
            None
        }
    }

    fn windows_to_iana(&self, tz: &str) -> String {
        match tz {
            "W. Europe Standard Time" => "Europe/Berlin".to_string(),
            "Central European Standard Time" => "Europe/Berlin".to_string(),
            "Central European Time" => "Europe/Berlin".to_string(),
            "Romance Standard Time" => "Europe/Paris".to_string(),
            "W. Central Africa Standard Time" => "Europe/Berlin".to_string(),
            "GTB Standard Time" => "Europe/Bucharest".to_string(),
            "Central Europe Standard Time" => "Europe/Budapest".to_string(),
            "Central Europe Time" => "Europe/Budapest".to_string(),
            "Morocco Standard Time" => "Africa/Casablanca".to_string(),
            "Greenwich Standard Time" => "Atlantic/Reykjavik".to_string(),
            "GMT Standard Time" => "Europe/London".to_string(),
            "UTC" => "UTC".to_string(),
            "UTC+12" => "Pacific/Auckland".to_string(),
            "UTC+13" => "Pacific/Tongatapu".to_string(),
            "Eastern Standard Time" => "America/New_York".to_string(),
            "Central Standard Time" => "America/Chicago".to_string(),
            "Mountain Standard Time" => "America/Denver".to_string(),
            "Pacific Standard Time" => "America/Los_Angeles".to_string(),
            "Alaska Standard Time" => "America/Anchorage".to_string(),
            "Hawaiian Standard Time" => "Pacific/Honolulu".to_string(),
            "SA Pacific Standard Time" => "America/Bogota".to_string(),
            "Eastern South America Standard Time" => "America/Sao_Paulo".to_string(),
            "Argentina Standard Time" => "America/Argentina/Buenos_Aires".to_string(),
            "Atlantic Standard Time" => "America/Halifax".to_string(),
            "Venezuela Standard Time" => "America/Caracas".to_string(),
            "Paraguay Standard Time" => "America/Asuncion".to_string(),
            "Central Brazilian Standard Time" => "America/Cuiaba".to_string(),
            "SA Western Standard Time" => "America/La_Paz".to_string(),
            "Pacific SA Standard Time" => "America/Santiago".to_string(),
            "E. South America Standard Time" => "America/Sao_Paulo".to_string(),
            "Tokyo Standard Time" => "Asia/Tokyo".to_string(),
            "Korea Standard Time" => "Asia/Seoul".to_string(),
            "China Standard Time" => "Asia/Shanghai".to_string(),
            "Singapore Standard Time" => "Asia/Singapore".to_string(),
            "Taipei Standard Time" => "Asia/Taipei".to_string(),
            "Hong Kong Standard Time" => "Asia/Hong_Kong".to_string(),
            "India Standard Time" => "Asia/Kolkata".to_string(),
            "Bangladesh Standard Time" => "Asia/Dhaka".to_string(),
            "Nepal Standard Time" => "Asia/Kathmandu".to_string(),
            "Sri Lanka Standard Time" => "Asia/Colombo".to_string(),
            "Myanmar Standard Time" => "Asia/Yangon".to_string(),
            "SE Asia Standard Time" => "Asia/Bangkok".to_string(),
            "W. Australia Standard Time" => "Australia/Perth".to_string(),
            "AUS Central Standard Time" => "Australia/Darwin".to_string(),
            "E. Australia Standard Time" => "Australia/Brisbane".to_string(),
            "AUS Eastern Standard Time" => "Australia/Sydney".to_string(),
            "Tasmania Standard Time" => "Australia/Hobart".to_string(),
            "Lord Howe Standard Time" => "Australia/Lord_Howe".to_string(),
            "New Zealand Standard Time" => "Pacific/Auckland".to_string(),
            "Fiji Standard Time" => "Pacific/Fiji".to_string(),
            "Arab Standard Time" => "Asia/Riyadh".to_string(),
            "Arabian Standard Time" => "Asia/Dubai".to_string(),
            "Iran Standard Time" => "Asia/Tehran".to_string(),
            "Israel Standard Time" => "Asia/Jerusalem".to_string(),
            "Turkey Standard Time" => "Europe/Istanbul".to_string(),
            "Egypt Standard Time" => "Africa/Cairo".to_string(),
            "South Africa Standard Time" => "Africa/Johannesburg".to_string(),
            "E. Africa Standard Time" => "Africa/Nairobi".to_string(),
            "Nigerian Standard Time" => "Africa/Lagos".to_string(),
            "Russia Time Zone 10" => "Asia/Srednekolymsk".to_string(),
            "Russia Time Zone 11" => "Asia/Kamchatka".to_string(),
            "Russia Time Zone 3" => "Europe/Volgograd".to_string(),
            "Russian Standard Time" => "Europe/Moscow".to_string(),
            "Syria Standard Time" => "Asia/Damascus".to_string(),
            "Jordan Standard Time" => "Asia/Amman".to_string(),
            "Middle East Standard Time" => "Asia/Beirut".to_string(),
            "Georgian Standard Time" => "Asia/Tbilisi".to_string(),
            "Azerbaijan Standard Time" => "Asia/Baku".to_string(),
            "Armenian Standard Time" => "Asia/Yerevan".to_string(),
            "Afghanistan Standard Time" => "Asia/Kabul".to_string(),
            "Pakistan Standard Time" => "Asia/Karachi".to_string(),
            "Uzbekistan Standard Time" => "Asia/Tashkent".to_string(),
            "West Asia Standard Time" => "Asia/Tashkent".to_string(),
            "Central Asia Standard Time" => "Asia/Almaty".to_string(),
            "North Asia Standard Time" => "Asia/Krasnoyarsk".to_string(),
            "N. Central Asia Standard Time" => "Asia/Novosibirsk".to_string(),
            "Tokyo Standard Time" => "Asia/Tokyo".to_string(),
            _ => tz.to_string(),
        }
    }

    fn timezone_to_location(&self, tz: &str) -> Option<Location> {
        let (lat, lon, city, country) = match tz {
            "Europe/Berlin" => (52.5200, 13.4050, "Berlin", "Germany"),
            "Europe/Vienna" => (48.2082, 16.3738, "Vienna", "Austria"),
            "Europe/Zurich" => (47.3769, 8.5417, "Zurich", "Switzerland"),
            "Europe/Amsterdam" => (52.3676, 4.9041, "Amsterdam", "Netherlands"),
            "Europe/Paris" => (48.8566, 2.3522, "Paris", "France"),
            "Europe/London" => (51.5074, -0.1278, "London", "United Kingdom"),
            "Europe/Rome" => (41.9028, 12.4964, "Rome", "Italy"),
            "Europe/Madrid" => (40.4168, -3.7038, "Madrid", "Spain"),
            "Europe/Prague" => (50.0755, 14.4378, "Prague", "Czech Republic"),
            "Europe/Warsaw" => (52.2297, 21.0122, "Warsaw", "Poland"),
            "Europe/Stockholm" => (59.3293, 18.0686, "Stockholm", "Sweden"),
            "Europe/Oslo" => (59.9139, 10.7522, "Oslo", "Norway"),
            "Europe/Copenhagen" => (55.6761, 12.5683, "Copenhagen", "Denmark"),
            "Europe/Helsinki" => (60.1699, 24.9384, "Helsinki", "Finland"),
            "Europe/Lisbon" => (38.7223, -9.1393, "Lisbon", "Portugal"),
            "Europe/Athens" => (37.9838, 23.7275, "Athens", "Greece"),
            "Europe/Bucharest" => (44.4268, 26.1025, "Bucharest", "Romania"),
            "Europe/Budapest" => (47.4979, 19.0402, "Budapest", "Hungary"),
            "Europe/Istanbul" => (41.0082, 28.9784, "Istanbul", "Turkey"),
            "Europe/Moscow" => (55.7558, 37.6173, "Moscow", "Russia"),
            "America/New_York" => (40.7128, -74.0060, "New York", "United States"),
            "America/Los_Angeles" => (34.0522, -118.2437, "Los Angeles", "United States"),
            "America/Chicago" => (41.8781, -87.6298, "Chicago", "United States"),
            "America/Denver" => (39.7392, -104.9903, "Denver", "United States"),
            "America/Phoenix" => (33.4484, -112.0740, "Phoenix", "United States"),
            "America/Toronto" => (43.6532, -79.3832, "Toronto", "Canada"),
            "America/Vancouver" => (49.2827, -123.1207, "Vancouver", "Canada"),
            "America/Sao_Paulo" => (-23.5505, -46.6333, "Sao Paulo", "Brazil"),
            "America/Mexico_City" => (19.4326, -99.1332, "Mexico City", "Mexico"),
            "America/Buenos_Aires" => (-34.6037, -58.3816, "Buenos Aires", "Argentina"),
            "America/Santiago" => (-33.4489, -70.6693, "Santiago", "Chile"),
            "America/Bogota" => (4.7110, -74.0721, "Bogota", "Colombia"),
            "America/Lima" => (-12.0464, -77.0428, "Lima", "Peru"),
            "Asia/Tokyo" => (35.6762, 139.6503, "Tokyo", "Japan"),
            "Asia/Shanghai" => (31.2304, 121.4737, "Shanghai", "China"),
            "Asia/Hong_Kong" => (22.3193, 114.1694, "Hong Kong", "Hong Kong"),
            "Asia/Singapore" => (1.3521, 103.8198, "Singapore", "Singapore"),
            "Asia/Seoul" => (37.5665, 126.9780, "Seoul", "South Korea"),
            "Asia/Taipei" => (25.0330, 121.5654, "Taipei", "Taiwan"),
            "Asia/Dubai" => (25.2048, 55.2708, "Dubai", "UAE"),
            "Asia/Mumbai" => (19.0760, 72.8777, "Mumbai", "India"),
            "Asia/Kolkata" => (22.5726, 88.3639, "Kolkata", "India"),
            "Asia/Bangkok" => (13.7563, 100.5018, "Bangkok", "Thailand"),
            "Asia/Jakarta" => (-6.2088, 106.8456, "Jakarta", "Indonesia"),
            "Asia/Manila" => (14.5995, 120.9842, "Manila", "Philippines"),
            "Asia/Riyadh" => (24.7136, 46.6753, "Riyadh", "Saudi Arabia"),
            "Asia/Tehran" => (35.6892, 51.3890, "Tehran", "Iran"),
            "Asia/Almaty" => (43.2220, 76.8512, "Almaty", "Kazakhstan"),
            "Australia/Sydney" => (-33.8688, 151.2093, "Sydney", "Australia"),
            "Australia/Melbourne" => (-37.8136, 144.9631, "Melbourne", "Australia"),
            "Australia/Brisbane" => (-27.4698, 153.0251, "Brisbane", "Australia"),
            "Australia/Perth" => (-31.9505, 115.8605, "Perth", "Australia"),
            "Pacific/Auckland" => (-36.8485, 174.7633, "Auckland", "New Zealand"),
            "Pacific/Fiji" => (-18.1248, 178.4501, "Fiji", "Fiji"),
            "Africa/Cairo" => (30.0444, 31.2357, "Cairo", "Egypt"),
            "Africa/Lagos" => (6.5244, 3.3792, "Lagos", "Nigeria"),
            "Africa/Johannesburg" => (-26.2041, 28.0473, "Johannesburg", "South Africa"),
            "Africa/Nairobi" => (-1.2921, 36.8219, "Nairobi", "Kenya"),
            "Africa/Casablanca" => (33.5731, -7.5898, "Casablanca", "Morocco"),
            _ => return None,
        };

        Some(
            Location::new(Coordinates::new(lat, lon), 50000.0, LocationSource::Timezone)
                .with_address(city, country, ""),
        )
    }
}

impl LocationProvider for TimezoneProvider {
    fn name(&self) -> &str {
        "Timezone"
    }

    fn is_available(&self) -> bool {
        self.get_system_timezone().is_some()
    }

    fn get_location(&self) -> Result<Location, LocationError> {
        let tz = self
            .get_system_timezone()
            .ok_or_else(|| LocationError::ProviderFailed("Could not detect timezone".into()))?;

        self.timezone_to_location(&tz)
            .ok_or_else(|| LocationError::ProviderFailed(format!("Unknown timezone: {}", tz)))
    }
}
