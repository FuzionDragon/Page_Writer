use dirs::home_dir;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

mod brain_compiler;
use brain_compiler::{submit_snippet, update_snippet};

use crate::brain_compiler::{sqlite_interface, CorpusSnippets, MarkedDocument};

const PATH: &str = "dev/rust/Page_Writer/src-tauri/src/data.db";

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

async fn setup_db() -> Result<SqlitePool, Error> {
    let path = home_dir()
        .expect("Unable to find home directory")
        .join(PATH)
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
async fn print(text: String) -> Result<(), Error> {
    println!("Printing {text}");

    Ok(())
}

#[tauri::command]
async fn fetch_marked_document() -> Result<Option<MarkedDocument>, Error> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            submit,
            update,
            load_snippets,
            mark_document,
            fetch_marked_document,
            print,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
