use anyhow::Ok;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub mod sqlite_interface;

pub type CorpusSnippets = HashMap<String, Vec<String>>;
pub type Corpus = HashMap<String, String>;

#[derive(Deserialize, Serialize)]
pub struct PageDocument {
    document_name: String,
    snippets: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct DocumentSnippets {
    document_id: i32,
    document_name: String,
    snippets: Vec<SnippetEntry>,
}

#[derive(Deserialize, Serialize)]
pub struct SnippetEntry {
    snippet_id: i32,
    snippet: String,
}

pub async fn submit_snippet(
    snippet: &str,
    title: Option<&str>,
    db: &SqlitePool,
) -> Result<(), anyhow::Error> {
    if snippet.is_empty() {
        println!("Snippet is empty");

        return Ok(());
    };

    if let Some(title) = title {
        sqlite_interface::add_document(db, title.trim(), snippet).await?;

        Ok(())
    } else {
        let marked_document = sqlite_interface::fetch_marked_document(db).await?;

        if let Some(marked_document) = marked_document {
            println!("Marked doc found");
            sqlite_interface::add_document(db, &marked_document, snippet).await?;

            return Ok(());
        }

        println!("No marked doc found");
        let first_line = snippet.lines().collect::<Vec<&str>>()[0];
        sqlite_interface::add_document(db, first_line.trim(), snippet).await?;
        sqlite_interface::set_marked_document(db, first_line.trim()).await?;

        Ok(())
    }
}

pub async fn update_snippet(
    db: &SqlitePool,
    snippet_id: i32,
    snippet: &str,
) -> Result<(), anyhow::Error> {
    if snippet.is_empty() {
        println!("Snippet is empty");
        sqlite_interface::delete_snippet(db, snippet_id).await?;
        return Ok(());
    };

    sqlite_interface::update_snippet(db, snippet_id, snippet).await?;

    Ok(())
}
