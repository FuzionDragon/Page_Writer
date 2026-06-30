use anyhow::{anyhow, Result};
use axum;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{prelude::*, BufReader, Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const PORT: &str = "55515";
const LINUX_TMP_DIR: &str = "/tmp";
const ANDROID_TMP_DIR: &str = "/data/local/tmp";
const LOCAL_DATA_NAME: &str = "local_data.db";
const REMOTE_DATA_NAME: &str = "remote_data.db";
const TIME_LIMIT: u64 = 20; // seconds

#[derive(Serialize, Deserialize, Debug)]
struct FileInformation {
    path: PathBuf,
    name: String,
    size_in_bytes: u64,
}

impl FileInformation {
    fn new(file_path: &str) -> Result<FileInformation> {
        let file = fs::File::open(file_path)?;
        let metadata = file.metadata()?;
        let path = PathBuf::from(file_path);
        let name = format!("{}", path.file_name().unwrap().to_string_lossy());
        println!("File name: {}", name);

        Ok(FileInformation {
            path,
            name,
            size_in_bytes: metadata.len(),
        })
    }

    fn file_data(&self) -> Result<Vec<u8>> {
        let mut file = fs::File::open(&self.path)?;
        let mut data = Vec::with_capacity(self.size_in_bytes as usize);
        file.read_to_end(&mut data)?;

        Ok(data)
    }
}

// needs to end the loop after a certain amount of time passes before declaring that no other valid
// peer is present.
fn server(data_path: &str) -> Result<String> {
    let server_address = format!("0.0.0.0:{}", PORT);
    let listener = TcpListener::bind(&server_address)?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Ok");
                let info = FileInformation::new(data_path)?;
                let bytes = serde_json::to_vec(&info).unwrap();
                stream.write_all(&bytes)?;
                let mut response = [0; 10];
                let amount_read = stream.read(&mut response)?;
                let response = String::from_utf8_lossy(&response[..amount_read]);
                let should_send = response == "YES";
                if should_send {
                    stream.write_all(&info.file_data()?)?;
                    return Ok("SUCCESS".to_string());
                }
            }
            Err(_err) => {
                println!("Server exists");
                return Ok("SERVER EXISTS".to_string());
            }
        }
    }

    println!("Reached end of server");
    Ok("NO SUCCESS".to_string())
}

// one direction, start from client to discover servers
fn udp_sender() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;

    let broadcast_addr = format!("255.255.255.255:{}", PORT); // Limited broadcast

    let msg = format!("{{\"type\":\"DISCOVERY\",\"seq\":1,\"service\":\"my-service\"}}",);
    socket.send_to(msg.as_bytes(), &broadcast_addr)?;

    //    for i in 0..3 {
    //        let msg = format!(
    //            "{{\"type\":\"DISCOVERY\",\"seq\":{},\"service\":\"my-service\"}}",
    //            i + 1
    //        );
    //        socket.send_to(msg.as_bytes(), &broadcast_addr)?;
    //        println!("Broadcast #{}: {}", i + 1, msg);
    //        thread::sleep(Duration::from_secs(1));
    //    }

    println!("Done broadcasting");
    Ok(())
}

// one direction, start from server to send IP to clients
// in the future may require encryption and other security measures
fn udp_reciever() -> Result<Option<String>> {
    let socket = UdpSocket::bind(&format!("0.0.0.0:{}", PORT))?;
    socket.set_broadcast(true)?;

    // 10-second receive timeout
    socket.set_read_timeout(Some(Duration::from_secs(10)))?;

    let mut buf = [0u8; 4096];
    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);

    loop {
        if start_time.elapsed() > time_limit {
            println!("Time out closed receiver");
            return Ok(None);
        }
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                let msg = std::str::from_utf8(&buf[..n]).unwrap_or("<invalid>");
                println!("From {}: {}", src, msg);

                // Send unicast reply back to the sender
                let reply = format!("{{\"type\":\"DISCOVERY_REPLY\",\"port\":8080}}");
                socket.send_to(reply.as_bytes(), src)?;

                return Ok(Some(src.to_string()));
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                println!("Timeout: no broadcasts received for 10 seconds");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
    Ok(None)
}

// used to find devices IPs on the network needed for completing data transfers
pub async fn search_devices() -> Result<String> {
    // first pass, checks if there is a server running by sending a broadcast,
    // expected to be returned if found
    println!("First send");
    //udp_sender()?;
    for _ in 0..3 {
        udp_sender()?;
        thread::sleep(Duration::from_secs(1));
    }

    // server thread will fail if there already exists a server with the same port and address on
    // the network
    let reciever_thread = thread::spawn(|| udp_reciever());

    //    let server_thread = thread::spawn(move || server(&owned_data_path));
    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);

    loop {
        if reciever_thread.is_finished() {
            println!("Thread is finished");
            let remote_ip = reciever_thread.join().unwrap()?;
            println!("Starting sender after joining threads");
            for _ in 0..3 {
                udp_sender()?;
                thread::sleep(Duration::from_secs(1));
            }

            println!("Sender completed");
            println!("Remote: {:?}", remote_ip);
            let result = match remote_ip {
                Some(result) => Ok(format!(
                    "IP found for existing client or server is found at {:?}",
                    result
                )),
                None => Ok(format!("No IP found")),
            };
            return result;
        }
        if start_time.elapsed() > time_limit {
            println!("Time out closed");
            return Ok("TIMEOUT".to_string());
        }
    }
}

// needs to start a server thread and also start a client thread using the other device thread
// may need to temporarily return a string result for Toast viewing
pub async fn share_data(data_path: &str, client_ip: String) -> Result<()> {
    // make a copy of local save
    #[cfg(target_os = "linux")]
    let path = Path::new(LINUX_TMP_DIR).join(LOCAL_DATA_NAME);

    #[cfg(target_os = "android")]
    let path = Path::new(ANDROID_TMP_DIR).join(LOCAL_DATA_NAME);

    let server_thread = thread::spawn(move || server(path.to_str().unwrap()));
    let client_thread = thread::spawn(move || client(client_ip).unwrap());
    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);

    loop {
        if client_thread.is_finished() && server_thread.is_finished() {
            println!("Server and client threads are done");
            break;
        }
        if start_time.elapsed() > time_limit {
            println!("Time out closed");
            break;
        }
    }

    Ok(())
}

// needs to take in the local ip of server device
fn client(client_ip: String) -> Result<()> {
    let client_address = format!("{}:{}", client_ip, PORT);
    //let client_address = format!("{}:{}", local_ip()?.to_string(), PORT);
    let mut stream = TcpStream::connect(client_address).unwrap();
    let mut buf = [0; 1024];
    let size = stream.read(&mut buf)?;
    let file_info: FileInformation = serde_json::from_slice(&buf[..size]).unwrap();
    println!("File information {:?}", file_info); //check to make sure it's the right file
    stream.write_all(b"YES")?; // respond to proceed the transfer
    println!("Downloading: {}", file_info.name);
    let mut file_data = Vec::new();
    loop {
        let mut buf = [0; 32768]; // increase buffer
        let size = stream.read(&mut buf)?;
        if size > 0 {
            file_data.extend_from_slice(&buf[..size]);
            let file_size = file_info.size_in_bytes;
            println!(
                "Received {} of {} bytes {:.3}%",
                file_data.len(),
                file_size,
                file_data.len() as f64 * 100.0 / file_size as f64
            );
        } else {
            //stream has been closed possibly due to completed transfer
            break;
        }
    }
    // save file
    // create the `test_folder` or specify your own folder
    #[cfg(target_os = "linux")]
    let path = Path::new(LINUX_TMP_DIR).join(file_info.name);

    #[cfg(target_os = "linux")]
    let remote_path = Path::new(LINUX_TMP_DIR).join(REMOTE_DATA_NAME);

    #[cfg(target_os = "android")]
    let path = Path::new(ANDROID_TMP_DIR).join(file_info.name);

    #[cfg(target_os = "android")]
    let remote_path = Path::new(ANDROID_TMP_DIR).join(REMOTE_DATA_NAME);

    let mut new = fs::File::create(&path).unwrap();
    new.write_all(&file_data)?;
    fs::rename(path, remote_path)?;
    println!("done");
    Ok(())
}
