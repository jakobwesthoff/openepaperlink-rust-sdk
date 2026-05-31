use openepaperlink_sdk::{Mac, UploadImageOptions};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: upload_image <AP_URL> <MAC> <IMAGE_PATH>");
        std::process::exit(1);
    }

    let client = openepaperlink_sdk::Client::builder(&args[1])
        .build()
        .expect("failed to build client");

    let mac: Mac = args[2].parse().expect("invalid MAC address");
    let image_path = &args[3];

    let image_bytes =
        std::fs::read(image_path).unwrap_or_else(|e| panic!("failed to read {image_path}: {e}"));

    println!(
        "Uploading {} ({} bytes) to tag {mac}...",
        image_path,
        image_bytes.len()
    );

    client
        .upload_image(&mac, image_bytes, &UploadImageOptions::default())
        .await
        .expect("failed to upload image");

    println!("Done.");
}
