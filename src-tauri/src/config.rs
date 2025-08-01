use anyhow::{Error, Ok};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

const PATH: &str = "dev/rust/Page_Writer/src-tauri/src/config.toml";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    cosine_weight: Option<f32>,
    threshold: Option<f32>,
    latest_bias: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Keybindings {
    pub switch_menu: Option<String>,
    pub submit_snippet: Option<String>,
    pub current_document_picker: Option<String>,
    pub marked_document_picker: Option<String>,
    pub delete_document_picker: Option<String>,
    pub delete_current_document: Option<String>,
    pub move_selected_snippet: Option<String>,
    pub delete_selected_snippet: Option<String>,
    pub update_selected_snippet: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    settings: Settings,
    keybindings: Keybindings,
}

async fn fetch_config() -> Result<Config, Error> {
    let path = home_dir()
        .expect("Unable to find home directory")
        .join(PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    let contents = fs::read_to_string(path)?;

    let config: Config = toml::from_str(&contents)?;

    println!("{:?}", config);

    Ok(config)
}

pub async fn fetch_keybindings() -> Result<Keybindings, Error> {
    Ok(fetch_config().await?.keybindings)
}

pub async fn fetch_settings() -> Result<Settings, Error> {
    Ok(fetch_config().await?.settings)
}
