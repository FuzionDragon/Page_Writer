use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;

use crate::{get_android_path, DATA_PATH};

fn serve() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;

    // Default
    #[cfg(target_os = "linux")]
    let path = dirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    #[cfg(target_os = "android")]
    let path = get_android_path()?;

    // Get local IP address
    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    println!("Server running on http://{}:8080", local_ip);
    println!("Share this URL with the receiver");
    println!("Waiting for connection...");

    // Accept one connection and then exit
    if let Ok((mut stream, addr)) = listener.accept() {
        println!("Connected from: {}", addr);

        let file_data = fs::read(path)?;

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
            file_data.len()
        );

        stream.write_all(response.as_bytes())?;
        stream.write_all(&file_data)?;
        println!("Database sent successfully!");
    }

    Ok(())
}
