use corelocation::{get_location, get_location_from, CoreLocation, LocationSource};
use std::time::Duration;

fn main() {
    println!("=== CoreLocation Test ===\n");

    println!("Verfügbare Provider:");
    let client = CoreLocation::new();
    for provider in client.available_providers() {
        println!("  - {}", provider);
    }

    println!("\n--- Test 1: Beste Location ---");
    match get_location() {
        Ok(loc) => {
            println!("Location: {}", loc);
            println!("Koordinaten: {}", loc.coordinates);
            println!("Genauigkeit: ±{:.0}m", loc.accuracy);
            println!("Quelle: {}", loc.source);
            if let Some(city) = &loc.city {
                println!("Stadt: {}", city);
            }
            if let Some(country) = &loc.country {
                println!("Land: {}", country);
            }
        }
        Err(e) => println!("Fehler: {}", e),
    }

    println!("\n--- Test 2: IP Location ---");
    match get_location_from(LocationSource::Ip) {
        Ok(loc) => println!("IP Location: {}", loc),
        Err(e) => println!("IP Fehler: {}", e),
    }

    println!("\n--- Test 3: Timezone Location ---");
    match get_location_from(LocationSource::Timezone) {
        Ok(loc) => println!("Timezone Location: {}", loc),
        Err(e) => println!("Timezone Fehler: {}", e),
    }

    println!("\n--- Test 4: GPS Location ---");
    match get_location_from(LocationSource::Gps) {
        Ok(loc) => println!("GPS Location: {}", loc),
        Err(e) => println!("GPS Fehler: {}", e),
    }

    println!("\n--- Test 5: Mit Cache ---");
    let cached_client = CoreLocation::new().with_cache_duration(Duration::from_secs(60));

    let start = std::time::Instant::now();
    let loc1 = cached_client.get_location();
    let duration1 = start.elapsed();

    let start = std::time::Instant::now();
    let loc2 = cached_client.get_location();
    let duration2 = start.elapsed();

    if let (Ok(l1), Ok(l2)) = (&loc1, &loc2) {
        println!("Erster Aufruf: {} ({:?})", l1, duration1);
        println!("Zweiter Aufruf (Cache): {} ({:?})", l2, duration2);
        println!("Cache speedup: {:?} schneller", duration1 - duration2);
    }

    println!("\n--- Test 6: Distanzberechnung ---");
    let berlin = corelocation::Coordinates::new(52.5200, 13.4050);
    let munich = corelocation::Coordinates::new(48.1351, 11.5820);
    let dist = berlin.distance_to(&munich);
    println!("Berlin → München: {:.0}km", dist / 1000.0);
}
