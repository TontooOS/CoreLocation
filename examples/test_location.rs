use corelocation::{get_location, get_location_from, CoreLocation, LocationSource};
use std::time::Duration;

fn main() {
    println!("=== CoreLocation Test ===\n");

    println!("Available providers:");
    let client = CoreLocation::new();
    for provider in client.available_providers() {
        println!("  - {}", provider);
    }

    println!("\n--- Test 1: Best location ---");
    match get_location() {
        Ok(loc) => {
            println!("Location: {}", loc);
            println!("Coordinates: {}", loc.coordinates);
            println!("Accuracy: ±{:.0}m", loc.accuracy);
            println!("Source: {}", loc.source);
            if let Some(city) = &loc.city {
                println!("City: {}", city);
            }
            if let Some(country) = &loc.country {
                println!("Country: {}", country);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n--- Test 2: WiFi location ---");
    match get_location_from(LocationSource::Wifi) {
        Ok(loc) => println!("WiFi Location: {}", loc),
        Err(e) => println!("WiFi error: {}", e),
    }

    println!("\n--- Test 3: IP location ---");
    match get_location_from(LocationSource::Ip) {
        Ok(loc) => println!("IP Location: {}", loc),
        Err(e) => println!("IP error: {}", e),
    }

    println!("\n--- Test 4: Timezone location ---");
    match get_location_from(LocationSource::Timezone) {
        Ok(loc) => println!("Timezone Location: {}", loc),
        Err(e) => println!("Timezone error: {}", e),
    }

    println!("\n--- Test 5: GPS location ---");
    match get_location_from(LocationSource::Gps) {
        Ok(loc) => println!("GPS Location: {}", loc),
        Err(e) => println!("GPS error: {}", e),
    }

    println!("\n--- Test 6: With cache ---");
    let cached_client = CoreLocation::new().with_cache_duration(Duration::from_secs(60));

    let start = std::time::Instant::now();
    let loc1 = cached_client.get_location();
    let duration1 = start.elapsed();

    let start = std::time::Instant::now();
    let loc2 = cached_client.get_location();
    let duration2 = start.elapsed();

    if let (Ok(l1), Ok(l2)) = (&loc1, &loc2) {
        println!("First call: {} ({:?})", l1, duration1);
        println!("Second call (cache): {} ({:?})", l2, duration2);
        println!("Cache speedup: {:?} faster", duration1 - duration2);
    }

    println!("\n--- Test 7: Distance calculation ---");
    let berlin = corelocation::Coordinates::new(52.5200, 13.4050);
    let munich = corelocation::Coordinates::new(48.1351, 11.5820);
    let dist = berlin.distance_to(&munich);
    println!("Berlin -> Munich: {:.0}km", dist / 1000.0);
}
