use openepaperlink_sdk::{StreamExt, WsMessage};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ws_monitor <AP_URL>");
        std::process::exit(1);
    }

    let client = openepaperlink_sdk::Client::builder(&args[1])
        .build()
        .expect("failed to build client");

    println!("Connecting to {}...", args[1]);
    let mut stream = client
        .connect_ws()
        .await
        .expect("failed to connect WebSocket");
    println!("Connected. Listening for events (Ctrl+C to stop)...\n");

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => match msg {
                WsMessage::SystemInfo(sys) => {
                    println!(
                        "[sys] heap={} dbsize={} tags={} uptime={}s rssi={}",
                        sys.heap, sys.dbsize, sys.recordcount, sys.uptime, sys.rssi
                    );
                }
                WsMessage::TagUpdate(tags) => {
                    for tag in &tags {
                        println!("[tag] {} alias={:?} mode={:?}", tag.mac, tag.alias, tag.content_mode);
                    }
                }
                WsMessage::ApItem(ap) => {
                    println!("[ap] {} alias={:?} tags={}", ap.ip, ap.alias, ap.count);
                }
                WsMessage::Log(msg) => println!("[log] {msg}"),
                WsMessage::Error(msg) => println!("[err] {msg}"),
                WsMessage::Console { text, color } => {
                    if let Some(color) = color {
                        println!("[console:{color}] {text}");
                    } else {
                        println!("[console] {text}");
                    }
                }
                _ => println!("[unknown] {msg:?}"),
            },
            Err(e) => {
                eprintln!("[error] {e}");
                break;
            }
        }
    }

    println!("WebSocket disconnected.");
}
