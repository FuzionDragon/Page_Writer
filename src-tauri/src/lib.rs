use anyhow::Result;
use std::fs::{self};

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

mod config;
mod data_exchange;
mod database;

use database::{submit_snippet, update_snippet};

use crate::{
    config::{Keybindings, Settings},
    database::{sqlite_interface, CorpusSnippets, DocumentSnippet},
};

// need to change path to prevent having multiple cache / storage directories on Linux
//pub const DATA_PATH: &str = "PageWriter/data.db";
//pub const BACKUP_DATA_PATH: &str = "PageWriter/data.db.bkp";

// paths hard coded with current tauri identifier, need to remember to change this in the future
pub const DATA_PATH: &str = "com.davidl.page-writer/data/data.db";
pub const BACKUP_DATA_PATH: &str = "com.davidl.page-writer/data/data.db.bkp";
const REMOTE_DATA_NAME: &str = "remote_data.db";
const ANDROID_APP_NAME: &str = "com.davidl.page_writer";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(target_os = "android")]
async fn prepare_android_paths() -> Result<()> {
    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join("com.davidl.page-writer/data")
        .into_os_string()
        .into_string()
        .unwrap();

    fs::create_dir_all(&path)?;

    Ok(())
}

async fn setup_db() -> Result<SqlitePool, Error> {
    let data_path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join("com.davidl.page-writer/data")
        .into_os_string()
        .into_string()
        .unwrap();

    fs::create_dir_all(&data_path)?;

    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    if !Sqlite::database_exists(&path).await.unwrap_or(false) {
        println!("Creating database: {}", &path);
        Sqlite::create_database(&path).await?;
    }

    let db = SqlitePool::connect(&path).await?;

    sqlite_interface::init(&db).await?;

    Ok(db)
}

async fn make_backup() -> Result<String, Error> {
    #[cfg(target_os = "android")]
    prepare_android_paths().await?;

    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    let backup_path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(BACKUP_DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    fs::copy(path, backup_path)?;

    Ok(String::new())
}

#[tauri::command]
fn get_config_path() -> Result<String, Error> {
    let path = config::fetch_config_path()?;

    Ok(path)
}

#[tauri::command]
fn get_android_path_tauri() -> Result<String, Error> {
    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    fs::create_dir_all(&path)?;

    Ok(path)
}

#[tauri::command]
async fn load_keybindings() -> Result<Option<Keybindings>, Error> {
    // Default
    #[cfg(target_os = "linux")]
    let keybindings = config::fetch_keybindings().await?;

    #[cfg(target_os = "android")]
    let keybindings = None;

    Ok(keybindings)
}

#[tauri::command]
async fn load_settings() -> Result<Option<Settings>, Error> {
    let settings = config::fetch_settings().await?;

    Ok(settings)
}

#[tauri::command]
async fn submit(snippet: String, title: String) -> Result<(), Error> {
    let db = setup_db().await?;

    if title.is_empty() {
        submit_snippet(&snippet, None, &db).await?;
    } else {
        submit_snippet(&snippet, Some(&title), &db).await?;
    }

    Ok(())
}

#[tauri::command]
async fn update(snippet_id: i32, snippet: String) -> Result<(), Error> {
    let db = setup_db().await?;

    update_snippet(&db, snippet_id, &snippet).await?;

    Ok(())
}

#[tauri::command]
async fn load_snippets() -> Result<CorpusSnippets, Error> {
    let db = setup_db().await?;

    let result = sqlite_interface::load_corpus_snippets(&db).await?;

    Ok(result)
}

#[tauri::command]
async fn load_document(document_name: String) -> Result<Option<DocumentSnippet>, Error> {
    let db = setup_db().await?;

    let result = sqlite_interface::fetch_document(&db, &document_name).await?;

    Ok(result)
}

// Only usable on PC, must have export path defined in toml, but default is in
// '~/Documents/PageWriter'
#[tauri::command]
async fn export_all_documents() -> Result<String, Error> {
    let db = setup_db().await?;

    let corpus = sqlite_interface::load_corpus(&db).await?;

    let mut document_path = sysdirs::document_dir().unwrap();
    document_path.push("PageWriter/");

    let path_string = document_path.into_os_string().into_string().unwrap();

    fs::create_dir(&path_string)?;

    for (name, contents) in &corpus {
        let file_path = format!("{path_string}{name}.md");

        fs::write(file_path, contents)?;
    }

    Ok(path_string)
}

#[tauri::command]
async fn print(text: String) -> Result<(), Error> {
    println!("Printing {text}");

    Ok(())
}

#[tauri::command]
async fn move_snippet(snippet_id: i32, document_name: String) -> Result<(), Error> {
    let db = setup_db().await?;

    sqlite_interface::move_snippet(&db, snippet_id, &document_name).await?;

    Ok(())
}

#[tauri::command]
async fn fetch_marked_document() -> Result<Option<String>, Error> {
    let db = setup_db().await?;

    let marked_document = sqlite_interface::fetch_marked_document(&db).await?;

    if let Some(marked_document) = marked_document {
        Ok(Some(marked_document))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn mark_document(document_name: String) -> Result<(), Error> {
    let db = setup_db().await?;

    sqlite_interface::set_marked_document(&db, &document_name).await?;

    Ok(())
}

#[tauri::command]
async fn delete_document(document_name: String) -> Result<(), Error> {
    let db = setup_db().await?;

    println!("{}", &document_name);
    sqlite_interface::delete_document(&db, &document_name).await?;

    Ok(())
}

#[tauri::command]
async fn delete_snippet(snippet_id: i32) -> Result<(), Error> {
    let db = setup_db().await?;

    sqlite_interface::delete_snippet(&db, snippet_id).await?;

    Ok(())
}

#[tauri::command]
async fn sync_data() -> Result<String, Error> {
    #[cfg(target_os = "android")]
    prepare_android_paths().await?;

    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    if !Sqlite::database_exists(&path).await.unwrap_or(false) {
        println!("Creating database: {}", &path);
        Sqlite::create_database(&path).await?;
    }

    let local_pool = setup_db().await?;

    make_backup().await?;

    let remote_path = sysdirs::temp_dir()
        .expect("Unable to find local data directory")
        .join(REMOTE_DATA_NAME)
        .into_os_string()
        .into_string()
        .unwrap();

    println!("Send data");
    let mut response = data_exchange::search_devices(path).await?;

    if !Sqlite::database_exists(&remote_path).await.unwrap_or(false) {
        println!("remote database is missing");
    } else {
        println!("remote database exists, syncing data");
        let remote_pool = SqlitePool::connect(&remote_path).await?;

        sqlite_interface::init(&local_pool).await?;
        sqlite_interface::init(&remote_pool).await?;

        database::sync_database(&local_pool, &remote_pool).await?;

        response = "Data copied successfully".to_string();
    }

    if fs::exists(&remote_path)? {
        fs::remove_file(remote_path)?;
    }

    Ok(response)
}

#[tauri::command]
fn load_backup() -> Result<String, Error> {
    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    let backup_path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(BACKUP_DATA_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    if fs::exists(&path)? && fs::exists(&backup_path)? {
        fs::copy(path, backup_path)?;
    } else {
        return Ok("Active or backup  database files missing".to_string());
    }

    Ok("Success".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config_path,
            get_android_path_tauri,
            load_keybindings,
            load_settings,
            submit,
            update,
            load_snippets,
            move_snippet,
            load_document,
            mark_document,
            fetch_marked_document,
            delete_document,
            delete_snippet,
            export_all_documents,
            sync_data,
            load_backup,
            print,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
