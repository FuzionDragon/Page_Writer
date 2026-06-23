use anyhow::{anyhow, Result};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{prelude::*, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

const PORT: &str = "55515";
const LINUX_TMP_DIR: &str = "/tmp";
const ANDROID_TMP_DIR: &str = "/data/local/tmp";
const TIME_LIMIT: u64 = 30; // seconds

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
fn server(data_path: &str, listener: TcpListener) -> Result<String> {
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

// needs to first try running a server command as a thread
pub async fn test(data_path: &str) -> Result<String> {
    let owned_data_path = data_path.to_owned();
    // check if address is in use
    let server_address = format!("0.0.0.0:{}", PORT);
    let server_listener = TcpListener::bind(&server_address);
    if server_listener.is_err() {
        let _client_thread = thread::spawn(|| client());
        return Ok("DEVICE FOUND".to_string());
    }

    let server_thread = thread::spawn(move || server(&owned_data_path, server_listener?));
    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);
    loop {
        if server_thread.is_finished() {
            println!("Thread is finished");
            let server_result = server_thread.join().unwrap()?;
            println!("{:?}", server_result);
            let result = match server_result.as_str() {
                "SUCCESS" => Ok("SUCCESS".to_string()),
                _ => Ok("NO SUCCESS".to_string()),
            };
            return result;
        }
        if start_time.elapsed() > time_limit {
            println!("Time out closed");
            return Ok("TIMEOUT".to_string());
        }
    }
}

fn client() -> Result<()> {
    let client_address = format!("{}:{}", local_ip()?.to_string(), PORT);
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
    let path = Path::new(LINUX_TMP_DIR).join(file_info.name);
    let mut new = fs::File::create(path).unwrap();
    new.write_all(&file_data)?;
    println!("done");
    Ok(())
}
