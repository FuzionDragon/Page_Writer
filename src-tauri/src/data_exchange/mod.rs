use anyhow::Result;
use std::{
    thread,
    time::{Duration, Instant},
};

const TIME_LIMIT: u64 = 10; // seconds

mod file_exchange;
use file_exchange::*;

// used to find devices IPs on the network needed for completing data transfers
pub async fn search_devices(data_path: String) -> Result<String> {
    //    return Ok(local_ip_address::local_ip().unwrap().to_string());
    // first pass, checks if there is a server running by sending a broadcast,
    // expected to be returned if found

    // server thread will fail if there already exists a server with the same port and address on
    // the network
    let path_clone = data_path.clone();
    let _server_thread = thread::spawn(move || tcp_server(&path_clone));
    let reciever_thread = thread::spawn(|| udp_reciever());
    let _udp_server_thread = thread::spawn(|| udp_sender());

    let start_time = Instant::now();
    let time_limit = Duration::new(TIME_LIMIT, 0);

    loop {
        // somewhere in this for if statement is the fault causing nothing to be returned on Android
        if reciever_thread.is_finished() {
            println!("Thread is finished");
            let remote_ip = reciever_thread.join().unwrap()?;
            println!("remote IP is {:?}", remote_ip);
            let remote_address: Option<String> = match remote_ip.clone() {
                Some(ip) => Some(ip.split(":").collect::<Vec<&str>>()[0].to_string()),
                None => None,
            };
            println!("Sending data to other device");

            // need to resolve 'temporarily stores borrowed value' nonsense
            if let Some(address) = remote_address {
                println!("Client address = {address}");
                // main fault it seems, potential panic in here
                tcp_client(address, &data_path)?;
            }

            let result = match remote_ip.clone() {
                Some(ip) => Ok(format!(
                    "Device found, data sent and recieved: {ip}" //           "IP found for existing client or server is found at {:?}",
                                                                 //           ip
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
