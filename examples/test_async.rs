use corelocation::{get_location_async, get_location_from_async, LocationSource};

#[tokio::main]
async fn main() {
    println!("=== CoreLocation Async Test ===\n");

    match get_location_async().await {
        Ok(loc) => println!("Best location: {}", loc),
        Err(e) => println!("Error: {}", e),
    }

    match get_location_from_async(LocationSource::Wifi).await {
        Ok(loc) => println!("WiFi location: {}", loc),
        Err(e) => println!("WiFi error: {}", e),
    }
}
