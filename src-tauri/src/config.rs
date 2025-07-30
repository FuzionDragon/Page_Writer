use anyhow::{Error, Ok};
use dirs::home_dir;
use std::fs;
use toml;

const PATH: &str = "dev/rust/Page_Writer/src-tauri/src/config.toml";

pub async fn fetch_config() -> Result<(), Error> {
    let path = home_dir()
        .expect("Unable to find home directory")
        .join(PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    let contents = fs::read_to_string(path)?;

    println!("{contents}");

    Ok(())
}
