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

// paths hard coded with current tauri identifier, need to remember to change this in the future
pub const DATA_PATH: &str = "com.davidl.page-writer/data/data.db";
pub const BACKUP_DATA_PATH: &str = "com.davidl.page-writer/data/data.db.bkp";
const REMOTE_DATA_NAME: &str = "remote_data.db";

//
// Unit Test Block
//
#[cfg(test)]
mod tests {
    const TEST_DATA_PATH: &str = "com.davidl.page-writer/data/test_data.db";
    const TEST_BACKUP_DATA_PATH: &str = "com.davidl.page-writer/data/test_data.db.bkp";

    use super::*;

    async fn test_cleanup() -> Result<()> {
        let data_path = sysdirs::data_local_dir()
            .expect("Unable to find local data directory")
            .join(TEST_DATA_PATH)
            .into_os_string()
            .into_string()
            .unwrap();

        if fs::exists(&data_path)? {
            //fs::remove_file(data_path)?;
        }

        let backup_path = sysdirs::data_local_dir()
            .expect("Unable to find local data directory")
            .join(TEST_BACKUP_DATA_PATH)
            .into_os_string()
            .into_string()
            .unwrap();

        if fs::exists(&backup_path)? {
            fs::remove_file(backup_path)?;
        }

        Ok(())
    }

    #[sqlx::test]
    async fn setup_db_test() -> Result<(), Error> {
        let db = setup_db(TEST_DATA_PATH).await?;

        assert!(sqlite_interface::validate_schema(&db).await?);

        let data_size = sqlite_interface::load_all_documents(&db).await?.len();
        assert_eq!(data_size, 0);

        test_cleanup().await?;

        Ok(())
    }

    #[sqlx::test]
    async fn create_document_test(db: SqlitePool) -> Result<(), Error> {
        sqlite_interface::init(&db).await?;

        sqlite_interface::add_document(&db, None, "Test", vec!["First Snippet".to_string()])
            .await?;

        let data = sqlite_interface::load_all_documents(&db).await?;

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].snippets.len(), 1);
        assert_eq!(data[0].snippets[0].snippet, "First Snippet".to_string());

        Ok(())
    }

    #[sqlx::test]
    async fn append_onto_document_test(db: SqlitePool) -> Result<(), Error> {
        sqlite_interface::init(&db).await?;

        sqlite_interface::add_document(&db, None, "Test", vec!["First Snippet".to_string()])
            .await?;

        sqlite_interface::add_document(&db, None, "Test", vec!["Second Snippet".to_string()])
            .await?;

        let data = sqlite_interface::load_all_documents(&db).await?;
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].snippets.len(), 2);

        for snippet in &data[0].snippets {
            if snippet.snippet != "First Snippet" && snippet.snippet != "Second Snippet" {
                panic!("Mismatched snippet name found.");
            }
        }

        Ok(())
    }

    #[sqlx::test]
    async fn fetch_document_test(db: SqlitePool) -> Result<(), Error> {
        sqlite_interface::init(&db).await?;

        sqlite_interface::add_document(&db, None, "Test", vec!["First Snippet".to_string()])
            .await?;

        let document = sqlite_interface::fetch_document(&db, "Test").await?;
        assert!(document.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_document_test(db: SqlitePool) -> Result<(), Error> {
        sqlite_interface::init(&db).await?;

        sqlite_interface::add_document(&db, None, "Test", vec!["First Snippet".to_string()])
            .await?;

        assert_eq!(sqlite_interface::load_all_documents(&db).await?.len(), 1);

        sqlite_interface::delete_document(&db, "Test").await?;

        assert_eq!(sqlite_interface::load_all_documents(&db).await?.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn fetch_nonexistent_document_test(db: SqlitePool) -> Result<(), Error> {
        sqlite_interface::init(&db).await?;

        let document = sqlite_interface::fetch_document(&db, "Test").await?;
        assert!(document.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn fetch_marked_document_test(db: SqlitePool) -> Result<(), Error> {
        sqlite_interface::init(&db).await?;

        assert!(sqlite_interface::fetch_marked_document(&db)
            .await?
            .is_none());

        sqlite_interface::add_document(&db, None, "Test", vec!["First Snippet".to_string()])
            .await?;

        sqlite_interface::set_marked_document(&db, "Test").await?;

        assert!(sqlite_interface::fetch_marked_document(&db)
            .await?
            .is_some());

        Ok(())
    }
}

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

async fn setup_db(data_path: &str) -> Result<SqlitePool, Error> {
    let data_path_dirs = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join("com.davidl.page-writer/data")
        .into_os_string()
        .into_string()
        .unwrap();

    fs::create_dir_all(&data_path_dirs)?;

    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(data_path)
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

async fn make_backup(data_path: &str, backup_data_path: &str) -> Result<String, Error> {
    #[cfg(target_os = "android")]
    prepare_android_paths().await?;

    let path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(data_path)
        .into_os_string()
        .into_string()
        .unwrap();

    let backup_path = sysdirs::data_local_dir()
        .expect("Unable to find local data directory")
        .join(backup_data_path)
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
    let db = setup_db(DATA_PATH).await?;

    if title.is_empty() {
        submit_snippet(&snippet, None, &db).await?;
    } else {
        submit_snippet(&snippet, Some(&title), &db).await?;
    }

    Ok(())
}

#[tauri::command]
async fn update(snippet_id: i32, snippet: String) -> Result<(), Error> {
    let db = setup_db(DATA_PATH).await?;

    update_snippet(&db, snippet_id, &snippet).await?;

    Ok(())
}

#[tauri::command]
async fn load_snippets() -> Result<CorpusSnippets, Error> {
    let db = setup_db(DATA_PATH).await?;

    let result = sqlite_interface::load_corpus_snippets(&db).await?;

    Ok(result)
}

#[tauri::command]
async fn load_document(document_name: String) -> Result<Option<DocumentSnippet>, Error> {
    let db = setup_db(DATA_PATH).await?;

    let result = sqlite_interface::fetch_document(&db, &document_name).await?;

    Ok(result)
}

// Only usable on PC so far, must have export path defined in toml, but default is in
// '~/Documents/PageWriter'
#[tauri::command]
async fn export_all_documents() -> Result<String, Error> {
    let db = setup_db(DATA_PATH).await?;

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
    let db = setup_db(DATA_PATH).await?;

    sqlite_interface::move_snippet(&db, snippet_id, &document_name).await?;

    Ok(())
}

#[tauri::command]
async fn fetch_marked_document() -> Result<Option<String>, Error> {
    let db = setup_db(DATA_PATH).await?;

    let marked_document = sqlite_interface::fetch_marked_document(&db).await?;

    if let Some(marked_document) = marked_document {
        Ok(Some(marked_document))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn mark_document(document_name: String) -> Result<(), Error> {
    let db = setup_db(DATA_PATH).await?;

    sqlite_interface::set_marked_document(&db, &document_name).await?;

    Ok(())
}

#[tauri::command]
async fn delete_document(document_name: String) -> Result<(), Error> {
    let db = setup_db(DATA_PATH).await?;

    println!("{}", &document_name);
    sqlite_interface::delete_document(&db, &document_name).await?;

    Ok(())
}

#[tauri::command]
async fn delete_snippet(snippet_id: i32) -> Result<(), Error> {
    let db = setup_db(DATA_PATH).await?;

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

    let local_pool = setup_db(DATA_PATH).await?;

    make_backup(DATA_PATH, BACKUP_DATA_PATH).await?;

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
