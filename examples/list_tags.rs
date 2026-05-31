use openepaperlink_sdk::{Battery, ContentMode, NextCheckin, Rssi};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: list_tags <AP_HOST>");
        std::process::exit(1);
    }

    let client = openepaperlink_sdk::Client::builder(&args[1])
        .build()
        .expect("failed to build client");

    let tags = client.get_tags().await.expect("failed to get tags");

    if tags.is_empty() {
        println!("No tags found.");
        return;
    }

    println!(
        "{:<18} {:<20} {:<6} {:<10} {:<8} {:<20}",
        "MAC", "Alias", "HW", "Battery", "RSSI", "Content Mode"
    );
    println!("{}", "-".repeat(90));

    for tag in &tags {
        let alias = if tag.alias.is_empty() {
            "-"
        } else {
            &tag.alias
        };

        let battery = match tag.battery {
            Battery::NotAvailable => "n/a".to_string(),
            Battery::Virtual => "virtual".to_string(),
            Battery::AtLeast(mv) => format!("≥{:.2}V", mv as f64 / 1000.0),
            Battery::Exact(mv) => format!("{:.2}V", mv as f64 / 1000.0),
            _ => "?".to_string(),
        };

        let rssi = match tag.rssi {
            Rssi::AccessPoint => "AP".to_string(),
            Rssi::Dbm(dbm) => format!("{dbm}dBm"),
            _ => "?".to_string(),
        };

        let mode = match tag.content_mode {
            ContentMode::None => "not configured",
            ContentMode::CurrentDate => "date",
            ContentMode::CurrentWeather => "weather",
            ContentMode::WeatherForecast => "forecast",
            ContentMode::JsonTemplate => "json template",
            ContentMode::StaticImage => "static image",
            ContentMode::ExternalImage => "external image",
            ContentMode::ApInfo => "AP info",
            ContentMode::RemoteContent => "remote",
            _ => "other",
        };

        let checkin = match tag.nextcheckin {
            NextCheckin::DeepSleep => " [deep sleep]".to_string(),
            NextCheckin::At(_) => String::new(),
            _ => String::new(),
        };

        println!(
            "{:<18} {:<20} 0x{:02X}  {:<10} {:<8} {}{checkin}",
            tag.mac, alias, tag.hw_type, battery, rssi, mode
        );
    }

    println!("\n{} tag(s) total.", tags.len());
}
