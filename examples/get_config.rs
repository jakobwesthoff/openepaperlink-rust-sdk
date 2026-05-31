#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: get_config <AP_URL>");
        std::process::exit(1);
    }

    let client = openepaperlink_sdk::Client::builder(&args[1])
        .build()
        .expect("failed to build client");

    println!("=== System Info ===");
    let sysinfo = client.get_sysinfo().await.expect("failed to get sysinfo");
    println!("  Firmware:    {}", sysinfo.buildversion);
    println!("  Build env:   {}", sysinfo.env);
    println!("  Build SHA:   {}", sysinfo.sha);
    println!("  PSRAM:       {} bytes", sysinfo.psramsize);
    println!("  Flash:       {} bytes", sysinfo.flashsize);
    println!("  AP version:  0x{:04X}", sysinfo.ap_version);
    println!("  Rollback:    {}", sysinfo.rollback);

    println!("\n=== AP Config ===");
    let config = client
        .get_ap_config()
        .await
        .expect("failed to get AP config");
    println!("  Alias:       {:?}", config.alias);
    println!("  Channel:     {}", config.channel);
    println!("  AP state:    {:?}", config.apstate);
    println!("  Max sleep:   {} min", config.maxsleep);
    println!("  Timezone:    {}", config.timezone);
    println!("  WiFi power:  {}", config.wifipower);

    println!("\n=== Capabilities ===");
    println!("  C6:          {}", config.has_c6);
    println!("  H2:          {}", config.has_h2);
    println!("  TLSR:        {}", config.has_tlsr);
    println!("  Flasher:     {}", config.has_flasher);
    println!("  BLE:         {}", config.has_ble_writer);
    println!("  Sub-GHz:     {}", config.has_sub_ghz);
    println!("  Save space:  {}", config.savespace);
}
