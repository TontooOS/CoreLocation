use corelocation::providers::wifi::{apple_wloc, mylnikov};

fn main() {
    println!("=== CoreLocation WiFi Backend Test ===\n");

    let known_bssid = "00:0C:42:1F:65:E9";

    println!("--- Test 1: Apple WLOC (known BSSID {}) ---", known_bssid);
    match apple_wloc::query(&[known_bssid.to_string()]) {
        Ok(results) => {
            if results.is_empty() {
                println!("No results (BSSID unknown to Apple)");
            }
            for ap in results.iter().take(10) {
                println!(
                    "  {} -> {:.6}, {:.6} (±{:.0}m)",
                    ap.bssid,
                    ap.latitude,
                    ap.longitude,
                    ap.accuracy_m.unwrap_or(0.0)
                );
            }
            println!("  Total results: {}", results.len());
        }
        Err(e) => println!("Apple WLOC error: {}", e),
    }

    println!("\n--- Test 2: Mylnikov single BSSID ---");
    match mylnikov::query_single(known_bssid) {
        Ok((lat, lon, range)) => println!("  {:.6}, {:.6} (±{:.0}m)", lat, lon, range),
        Err(e) => println!("Mylnikov error: {}", e),
    }

    println!("\n--- Test 3: Mylnikov multi BSSID ---");
    let pairs = vec![
        ("28:28:5d:d6:39:8a".to_string(), -76),
        ("90:94:e4:ac:12:26".to_string(), -80),
    ];
    match mylnikov::query_multi(&pairs) {
        Ok((lat, lon, range)) => println!("  {:.6}, {:.6} (±{:.0}m)", lat, lon, range),
        Err(e) => println!("Mylnikov multi error: {}", e),
    }

    println!("\n--- Test 4: Nominatim reverse geocoding ---");
    let coords = corelocation::Coordinates::new(52.5200, 13.4050);
    match corelocation::geocode::reverse_geocode(coords) {
        Ok(addr) => println!("  Berlin coords -> {}, {}, {}", addr.city, addr.region, addr.country),
        Err(e) => println!("Nominatim error: {}", e),
    }
}
