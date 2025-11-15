use std::collections::HashMap;

use anyhow::{Ok, Result};
use sqlx::{FromRow, SqlitePool};

use crate::brain_compiler::{DocumentSnippets, SnippetEntry};

use super::{Corpus, CorpusSnippets};

#[derive(Debug, FromRow, Clone)]
pub struct Snippet {
    snippet_id: i32,
    snippet: String,
    document_id: i32,
    document_name: String,
}

#[derive(Debug, FromRow, Clone)]
struct Document {
    document_id: i32,
    document_name: String,
}

pub async fn init(db: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
    CREATE TABLE IF NOT EXISTS Document (
      document_id INTEGER PRIMARY KEY AUTOINCREMENT,
      document_name TEXT UNIQUE,
      is_marked INTEGER
    );
    "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
  CREATE TABLE IF NOT EXISTS Snippet (
    snippet_id INTEGER PRIMARY KEY AUTOINCREMENT,
    snippet TEXT NOT NULL,
    document_id INTEGER NOT NULL,
    FOREIGN KEY (document_id)
      REFERENCES Document (document_id)
  );
  "#,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn load_corpus_snippets(db: &SqlitePool) -> Result<CorpusSnippets> {
    let snippets = sqlx::query_as::<_, Snippet>(
        r#"
    SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
    JOIN Document ON Snippet.document_id == Document.document_id;
  "#,
    )
    .fetch_all(db)
    .await?;

    let mut corpus_snippets: CorpusSnippets = HashMap::new();
    for snippet in snippets {
        corpus_snippets
            .entry(snippet.document_name)
            .or_default()
            .push(snippet.snippet);
    }

    Ok(corpus_snippets)
}

pub async fn load_corpus(db: &SqlitePool) -> Result<Corpus> {
    let corpus: Corpus = load_corpus_snippets(db)
        .await?
        .into_iter()
        .map(|(k, v)| {
            let val = v.join("\n\n");
            (k, val)
        })
        .collect();

    Ok(corpus)
}

pub async fn load_snippets(db: &SqlitePool, document_id: i32) -> Result<Vec<Snippet>> {
    let snippets = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.document_id = $1;
      "#,
    )
    .bind(document_id)
    .fetch_all(db)
    .await?;

    Ok(snippets)
}

pub async fn fetch_document(
    db: &SqlitePool,
    document_name: &str,
) -> Result<Option<DocumentSnippets>> {
    let snippets = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.document_name = $1;
      "#,
    )
    .bind(document_name)
    .fetch_all(db)
    .await?;

    if snippets.is_empty() {
        Ok(None)
    } else {
        let all_snippets: Vec<SnippetEntry> = snippets
            .iter()
            .map(|v| SnippetEntry {
                snippet_id: v.snippet_id,
                snippet: v.snippet.clone(),
            })
            .collect();
        let page_document = DocumentSnippets {
            document_id: snippets[0].document_id,
            document_name: snippets[0].document_name.clone(),
            snippets: all_snippets,
        };
        Ok(Some(page_document))
    }
}

pub async fn delete_snippet(db: &SqlitePool, snippet_id: i32) -> Result<()> {
    let document_id = sqlx::query_as::<_, Snippet>(
        r#"
    SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
    JOIN Document ON Snippet.document_id == Document.document_id
    WHERE snippet_id = $1;
    "#,
    )
    .bind(snippet_id)
    .fetch_one(db)
    .await?
    .document_id;

    sqlx::query("DELETE FROM Snippet WHERE snippet_id = $1")
        .bind(snippet_id)
        .execute(db)
        .await?;

    let is_empty = sqlx::query_as::<_, Snippet>(
        r#"
          SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
          JOIN Document ON Snippet.document_id == Document.document_id
          WHERE Document.document_id = $1;
        "#,
    )
    .bind(document_id)
    .fetch_all(db)
    .await?
    .is_empty();

    if is_empty {
        sqlx::query("DELETE FROM Document WHERE document_id = $1;")
            .bind(document_id)
            .execute(db)
            .await?;
    }

    Ok(())
}

pub async fn delete_document(db: &SqlitePool, document_name: &str) -> Result<()> {
    let snippets = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.document_name = $1;
      "#,
    )
    .bind(document_name)
    .fetch_all(db)
    .await?;

    for snippet in snippets {
        delete_snippet(db, snippet.snippet_id).await?;
    }

    Ok(())
}

pub async fn update_snippet(db: &SqlitePool, snippet_id: i32, new_snippet: &str) -> Result<()> {
    sqlx::query("UPDATE Snippet SET snippet = $1 WHERE snippet_id = $2;")
        .bind(new_snippet)
        .bind(snippet_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn move_snippet(db: &SqlitePool, snippet_id: i32, document_name: &str) -> Result<()> {
    println!("{snippet_id}");
    println!("{document_name}");
    let document_row = sqlx::query_as::<_, Document>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name)
    .fetch_one(db)
    .await?;

    sqlx::query("UPDATE Snippet SET document_id = $1 WHERE snippet_id = $2;")
        .bind(document_row.document_id)
        .bind(snippet_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn set_marked_document(db: &SqlitePool, document_name: &str) -> Result<()> {
    sqlx::query("UPDATE Document SET is_marked = NULL WHERE is_marked = 1;")
        .execute(db)
        .await?;

    sqlx::query("UPDATE Document SET is_marked = 1 WHERE document_name = $1;")
        .bind(document_name)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn fetch_marked_document(db: &SqlitePool) -> Result<Option<String>> {
    let marked_document = sqlx::query_as::<_, Document>(
        r#"
        SELECT document_id, document_name FROM Document
        WHERE is_marked = 1;
      "#,
    )
    .fetch_optional(db)
    .await?;

    if let Some(marked_document) = marked_document {
        Ok(Some(marked_document.document_name))
    } else {
        Ok(None)
    }
}

pub async fn add_snippet(db: &SqlitePool, snippet: &str, document_name: &str) -> Result<()> {
    println!("fetching document");
    let document_row = sqlx::query_as::<_, Document>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name)
    .fetch_one(db)
    .await?;

    sqlx::query("INSERT INTO Snippet (snippet, document_id) VALUES ($1, $2) ON CONFLICT(snippet, document_id) DO NOTHING;")
    .bind(snippet)
    .bind(document_row.document_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn add_document(db: &SqlitePool, document_name: &str, snippet: &str) -> Result<()> {
    println!("Querying docs");
    let document = sqlx::query_as::<_, Document>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name.trim())
    .fetch_optional(db)
    .await?;

    if let Some(document) = document {
        println!("Document with the corresponding title already exists");

        add_snippet(db, snippet, &document.document_name).await?;
        println!("Appended snippet to existing doc");

        return Ok(());
    }

    sqlx::query(
        "INSERT INTO Document (document_name) VALUES ($1) ON CONFLICT(document_name) DO NOTHING;",
    )
    .bind(document_name.trim())
    .execute(db)
    .await?;

    add_snippet(db, snippet, document_name.trim()).await?;

    Ok(())
}
