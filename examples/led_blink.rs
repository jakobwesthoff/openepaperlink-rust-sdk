use openepaperlink_sdk::{LedFlashPattern, Mac};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: led_blink <AP_HOST> <MAC> [stop]");
        std::process::exit(1);
    }

    let client = openepaperlink_sdk::Client::builder(&args[1])
        .build()
        .expect("failed to build client");

    let mac: Mac = args[2].parse().expect("invalid MAC address");
    let stop = args.get(3).is_some_and(|a| a == "stop");

    if stop {
        println!("Stopping LED on {mac}...");
        client
            .led_flash_stop(&mac)
            .await
            .expect("failed to stop LED");
    } else {
        // Green, red, blue cycling pattern: 2ms flash duration, 3 flashes
        // per color, 1s delay between colors, 2 full cycles.
        let pattern = LedFlashPattern::from_hex("213C530A20530A03530A0200")
            .expect("invalid pattern");
        println!("Blinking LED on {mac}...");
        client
            .led_flash(&mac, &pattern)
            .await
            .expect("failed to flash LED");
    }

    println!("Done.");
}
