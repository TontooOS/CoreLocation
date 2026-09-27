use corelocation::{get_location_async, get_location_from_async, LocationSource};
use foundation::async_runtime::RuntimeBuilder;

fn main() {
    let rt = RuntimeBuilder::new_multi_thread().build().unwrap();
    rt.block_on(async {
        println!("=== CoreLocation Async Test ===\n");

        match get_location_async().await {
            Ok(loc) => println!("Best location: {}", loc),
            Err(e) => println!("Error: {}", e),
        }

        match get_location_from_async(LocationSource::Wifi).await {
            Ok(loc) => println!("WiFi location: {}", loc),
            Err(e) => println!("WiFi error: {}", e),
        }
    });
}
