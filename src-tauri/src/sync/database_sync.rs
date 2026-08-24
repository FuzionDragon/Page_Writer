use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use sysdirs;

const PORT: &str = "55515";
const LOCAL_DATA_NAME: &str = "local_data.db";
const REMOTE_DATA_NAME: &str = "remote_data.db";
const TIME_LIMIT: u64 = 10; // seconds
                            //
const LINUX_TMP_DIR: &str = "/tmp";
// seems android doesn't have a temp dir when looking at modern version of Android, so manual
// deletion of these files are needed
//
// this is appended onto the usual data directory
const ANDROID_TMP_DIR: &str = "cache";

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
pub fn tcp_server(data_path: &str) -> Result<String> {
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

    println!("Reached end of tcp_server");
    Ok("NO SUCCESS".to_string())
}

// one direction, start from client to discover servers
pub fn udp_sender() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;

    let broadcast_addr = format!("255.255.255.255:{}", PORT); // Limited broadcast

    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);
    let msg = format!("{{\"type\":\"DISCOVERY\",\"seq\":1,\"service\":\"my-service\"}}",);

    loop {
        if start_time.elapsed() > time_limit {
            println!("Time out closed receiver");
            break;
        }
        socket.send_to(msg.as_bytes(), &broadcast_addr)?;
        thread::sleep(Duration::from_secs(1));
    }

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

// one direction, start from tcp_server to send IP to clients
// in the future may require encryption and other security measures
//
// needs to check if the address is the same as current device, otherwise continue loop
pub fn udp_reciever() -> Result<Option<String>> {
    println!();
    let socket = UdpSocket::bind(&format!("0.0.0.0:{}", PORT))?;
    socket.set_broadcast(true)?;

    // 10-second receive timeout
    socket.set_read_timeout(Some(Duration::from_secs(10)))?;

    let mut buf = [0u8; 4096];
    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);

    loop {
        if start_time.elapsed() > time_limit {
            // main fault it seems, potential panic in here
            println!("Time out closed receiver");
            return Ok(None);
        }
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                if src.to_string().split(":").collect::<Vec<&str>>()[0]
                    != local_ip_address::local_ip()?.to_string()
                {
                    println!("Caught different device IP: {src}");

                    let msg = std::str::from_utf8(&buf[..n]).unwrap_or("<invalid>");
                    println!("From {}: {}", src, msg);

                    // Send unicast reply back to the sender
                    let reply = format!("{{\"type\":\"DISCOVERY_REPLY\",\"port\":8080}}");
                    socket.send_to(reply.as_bytes(), src)?;

                    return Ok(Some(src.to_string()));
                } else {
                    println!(
                        "Caught broadcast from same IP {}, continuing loop",
                        src.to_string()
                    );
                }
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

// needs to take in the local ip of tcp_server device
//
// potential the function that causes the breakage
pub fn tcp_client(client_ip: String, data_path: &str) -> Result<()> {
    let client_address = format!("{}:{}", client_ip, PORT);
    //let client_address = format!("{}:{}", local_ip()?.to_string(), PORT);
    // potential panic?
    let mut stream = TcpStream::connect(client_address)?;
    let mut buf = [0; 1024];
    let size = stream.read(&mut buf)?;
    let file_info: FileInformation = serde_json::from_slice(&buf[..size])?;
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

    let tmp_dir = sysdirs::temp_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    #[cfg(target_os = "android")]
    fs::create_dir_all(&tmp_dir)?;

    // save file
    // create the `test_folder` or specify your own folder
    let path = Path::new(&tmp_dir).join(file_info.name);

    let remote_path = Path::new(&tmp_dir).join(REMOTE_DATA_NAME);

    let local_path = Path::new(&tmp_dir).join(LOCAL_DATA_NAME);

    // temporary
    // fault lies here, not a directory error found, meaning that the tmp directory may not exist
    //    println!("Android {:?}", prepare_android_tmp_path(data_path));
    //    #[cfg(target_os = "android")]
    //    let path = Path::new(data_path).join(file_info.name);
    //
    //    #[cfg(target_os = "android")]
    //    let remote_path = Path::new(data_path)
    //        .join(ANDROID_TMP_DIR)
    //        .join(REMOTE_DATA_NAME);
    //
    //    #[cfg(target_os = "android")]
    //    let local_path = Path::new(data_path)
    //        .join(ANDROID_TMP_DIR)
    //        .join(LOCAL_DATA_NAME);

    let mut new = fs::File::create(&path)?;
    new.write_all(&file_data)?;
    fs::rename(path, remote_path)?;
    println!("done saving remote file");

    fs::copy(data_path, local_path)?;
    println!("local data copied to temp");

    Ok(())
}

//#[cfg(target_os = "android")]
fn prepare_android_tmp_path(data_path: &str) -> Result<PathBuf> {
    let mut path = PathBuf::from(data_path);
    path.pop();
    path.push(ANDROID_TMP_DIR);

    Ok(path)
}
