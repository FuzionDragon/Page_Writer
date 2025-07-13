#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use dirs::home_dir;

mod brain_compiler;
use brain_compiler::submit_snippet;

const PATH: &str = "dev/rust/page_compiler_tauri/src/data.db";

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

#[tauri::command]
async fn page_compiler_entry(snippet: String) -> Result<(), Error>{
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

  submit_snippet(&snippet, &db).await?;

  Ok(())
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![page_compiler_entry])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
