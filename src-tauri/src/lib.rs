use std::fs;

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

mod brain_compiler;
mod config;
use brain_compiler::{submit_snippet, update_snippet};

use crate::{
    brain_compiler::{sqlite_interface, CorpusSnippets, DocumentSnippets},
    config::{Keybindings, Settings},
};

const DATA_PATH: &str = "PageWriter/data.db";
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

fn get_android_path() -> Result<String, Error> {
    let data_path = format!("{}/{}/{}", "/data/data", ANDROID_APP_NAME, "files");
    fs::create_dir_all(&data_path)?;

    Ok(format!("{}/data.db", data_path))
}

async fn setup_db() -> Result<SqlitePool, Error> {
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

    if !Sqlite::database_exists(&path).await.unwrap_or(false) {
        println!("Creating database: {}", &path);
        Sqlite::create_database(&path).await?;
    }

    let db = SqlitePool::connect(&path).await?;

    sqlite_interface::init(&db).await?;

    Ok(db)
}

#[tauri::command]
fn get_config_path() -> Result<String, Error> {
    let path = config::fetch_config_path()?;

    Ok(path)
}

#[tauri::command]
fn get_android_path_tauri() -> Result<String, Error> {
    let path = get_android_path()?;

    Ok(path)
}

#[tauri::command]
async fn load_keybindings() -> Result<Option<Keybindings>, Error> {
    let keybindings = config::fetch_keybindings().await?;

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
async fn load_document(document_name: String) -> Result<Option<DocumentSnippets>, Error> {
    let db = setup_db().await?;

    let result = sqlite_interface::fetch_document(&db, &document_name).await?;

    Ok(result)
}

#[tauri::command]
async fn print(text: String) -> Result<(), Error> {
    println!("Printing {text}");

    Ok(())
}

#[tauri::command]
async fn move_snippet(snippet_id: i32, document_id: i32) -> Result<(), Error> {
    let db = setup_db().await?;

    sqlite_interface::move_snippet(&db, snippet_id, document_id).await?;

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
            print,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
