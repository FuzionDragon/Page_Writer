use anyhow::{Error, Ok};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const CONFIGPATH: &str = "PageWriter/config.toml";
const ANDROID_APP_NAME: &str = "com.davidl.page_writer";

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

fn get_android_path() -> String {
    format!(
        "{}/{}/{}",
        "/data/data", ANDROID_APP_NAME, "files/config.toml"
    )
}

pub fn fetch_config_path() -> Result<String, Error> {
    // Default
    #[cfg(target_os = "linux")]
    let path = dirs::config_local_dir()
        .expect("Unable to find local config directory")
        .join(CONFIGPATH)
        .into_os_string()
        .into_string()
        .unwrap();

    #[cfg(target_os = "android")]
    let path = get_android_path();

    Ok(path)
}

async fn fetch_config() -> Result<Option<Config>, Error> {
    // Default
    #[cfg(target_os = "linux")]
    let path = dirs::config_local_dir()
        .expect("Unable to find local config directory")
        .join(CONFIGPATH)
        .into_os_string()
        .into_string()
        .unwrap();

    #[cfg(target_os = "android")]
    let path = get_android_path();

    if !Path::new(&path).exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)?;

    let config: Config = toml::from_str(&contents)?;

    println!("{:?}", config);

    Ok(Some(config))
}

pub async fn fetch_keybindings() -> Result<Option<Keybindings>, Error> {
    let config = fetch_config().await?;

    if let Some(config) = config {
        Ok(Some(config.keybindings))
    } else {
        Ok(None)
    }
}

pub async fn fetch_settings() -> Result<Option<Settings>, Error> {
    let config = fetch_config().await?;

    if let Some(config) = config {
        Ok(Some(config.settings))
    } else {
        Ok(None)
    }
}
