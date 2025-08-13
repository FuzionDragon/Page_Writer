use std::collections::HashMap;

use anyhow::{Ok, Result};
use sqlx::{FromRow, SqlitePool};

use crate::brain_compiler::{DocumentSnippets, SnippetEntry};

use super::{Corpus, CorpusSnippets, PageDocument};

#[derive(Debug, FromRow, Clone)]
pub struct Snippet {
    snippet_id: i32,
    snippet: String,
    document_id: i32,
    document_name: String,
}

#[derive(Debug, FromRow, Clone)]
pub struct Phrase {
    phrase: String,
    document_name: String,
}

#[derive(Debug, FromRow, Clone)]
pub struct Term {
    term: String,
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
      is_last INTEGER,
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
    snippet TEXT NOT NULL UNIQUE,
    document_id INTEGER NOT NULL,
    UNIQUE (snippet, document_id),
    FOREIGN KEY (document_id)
      REFERENCES Document (document_id)
  );
  "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
    CREATE TABLE IF NOT EXISTS TFIDF_Term (
      term TEXT NOT NULL,
      snippet_id INTEGER NOT NULL,
      PRIMARY KEY (term, snippet_id),
      FOREIGN KEY (snippet_id)
        REFERENCES Snippet (snippet_id)
    );
    "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
      CREATE TABLE IF NOT EXISTS RAKE_Phrase (
        phrase TEXT NOT NULL,
        snippet_id INTEGER NOT NULL,
        PRIMARY KEY (phrase, snippet_id),
        FOREIGN KEY (snippet_id)
          REFERENCES Snippet (snippet_id)
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

pub async fn load_tfidf_data(db: &SqlitePool) -> Result<CorpusSnippets> {
    let terms = sqlx::query_as::<_, Term>(
        r#"
    SELECT term, Document.document_name FROM TFIDF_Term
    JOIN Snippet ON Snippet.snippet_id = TFIDF_Term.snippet_id
    JOIN Document ON Document.document_id = Snippet.document_id
    GROUP BY term;
  "#,
    )
    .fetch_all(db)
    .await?;

    let mut corpus_terms: CorpusSnippets = HashMap::new();
    for term in terms {
        corpus_terms
            .entry(term.document_name)
            .or_default()
            .push(term.term);
    }

    Ok(corpus_terms)
}

pub async fn load_rake_data(db: &SqlitePool) -> Result<CorpusSnippets> {
    let phrases = sqlx::query_as::<_, Phrase>(
        r#"
    SELECT phrase, Document.document_name FROM RAKE_Phrase
    JOIN Snippet ON Snippet.snippet_id = RAKE_Phrase.snippet_id
    JOIN Document ON Document.document_id = Snippet.document_id
    GROUP BY phrase;
  "#,
    )
    .fetch_all(db)
    .await?;

    let mut corpus_phrases: CorpusSnippets = HashMap::new();
    for phrase in phrases {
        corpus_phrases
            .entry(phrase.document_name)
            .or_default()
            .push(phrase.phrase);
    }

    Ok(corpus_phrases)
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

    sqlx::query("DELETE FROM TFIDF_Term WHERE snippet_id = $1")
        .bind(snippet_id)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM RAKE_Phrase WHERE snippet_id = $1")
        .bind(snippet_id)
        .execute(db)
        .await?;
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

pub async fn update_snippet(
    db: &SqlitePool,
    snippet_id: i32,
    new_snippet: &str,
    terms: Vec<String>,
    phrases: Vec<String>,
) -> Result<()> {
    sqlx::query("DELETE FROM TFIDF_Term WHERE snippet_id = $1;")
        .bind(snippet_id)
        .execute(db)
        .await?;

    sqlx::query("DELETE FROM RAKE_Phrase WHERE snippet_id = $1;")
        .bind(snippet_id)
        .execute(db)
        .await?;

    sqlx::query("UPDATE Snippet SET snippet = $1 WHERE snippet_id = $2;")
        .bind(new_snippet)
        .bind(snippet_id)
        .execute(db)
        .await?;

    let snippet = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE snippet_id = $1;
      "#,
    )
    .bind(snippet_id)
    .fetch_one(db)
    .await?;

    update_tfidf_data(db, terms, snippet.document_id).await?;
    update_rake_data(db, phrases, snippet.document_id).await?;

    Ok(())
}

pub async fn update_tfidf_data(
    db: &SqlitePool,
    terms: Vec<String>,
    document_id: i32,
) -> Result<()> {
    let snippet = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.document_id = $1;
      "#,
    )
    .bind(document_id)
    .fetch_one(db)
    .await?;

    for term in terms {
        sqlx::query("INSERT OR IGNORE INTO TFIDF_Term (term, snippet_id) VALUES ($1, $2) ON CONFLICT(term, snippet_id) DO NOTHING;")
      .bind(term)
      .bind(snippet.snippet_id)
      .execute(db)
      .await?;
    }

    Ok(())
}

pub async fn update_rake_data(
    db: &SqlitePool,
    phrases: Vec<String>,
    document_id: i32,
) -> Result<()> {
    let snippet = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_id, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.document_id = $1;
      "#,
    )
    .bind(document_id)
    .fetch_one(db)
    .await?;

    for phrase in phrases {
        sqlx::query("INSERT OR IGNORE INTO RAKE_Phrase (phrase, snippet_id) VALUES ($1, $2) ON CONFLICT(phrase, snippet_id) DO NOTHING;")
      .bind(phrase)
      .bind(snippet.snippet_id)
      .execute(db)
      .await?;
    }

    Ok(())
}

pub async fn move_snippet(db: &SqlitePool, snippet_id: i32, document_id: i32) -> Result<()> {
    sqlx::query("UPDATE Snippet SET document_id = $1 WHERE snippet_id = $2;")
        .bind(document_id)
        .bind(snippet_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn set_marked_document(db: &SqlitePool, document_id: i32) -> Result<()> {
    sqlx::query("UPDATE Document SET is_marked = NULL WHERE is_marked = 1;")
        .execute(db)
        .await?;

    sqlx::query("UPDATE Document SET is_marked = 1 WHERE document_id = $1;")
        .bind(document_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn fetch_marked_document(db: &SqlitePool) -> Result<Option<DocumentSnippets>> {
    let snippets = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT snippet_id, snippet, Document.document_name FROM Snippet
        JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.is_marked = 1;
      "#,
    )
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

pub async fn set_latest_document(db: &SqlitePool, document_id: i32) -> Result<()> {
    sqlx::query("UPDATE Document SET is_last = NULL WHERE is_last = 1;")
        .execute(db)
        .await?;

    sqlx::query("UPDATE Document SET is_last = 1 WHERE document_id = $1;")
        .bind(document_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn fetch_latest_document(db: &SqlitePool) -> Result<Option<PageDocument>> {
    let snippets = sqlx::query_as::<_, Snippet>(
        r#"
        SELECT Document.document_name, snippet FROM Snippet
        LEFT JOIN Document ON Snippet.document_id == Document.document_id
        WHERE Document.is_last = 1;
      "#,
    )
    .fetch_all(db)
    .await?;

    if snippets.is_empty() {
        Ok(None)
    } else {
        let all_snippets: Vec<String> = snippets.iter().map(|v| v.snippet.clone()).collect();
        let page_document = PageDocument {
            document_name: snippets[0].document_name.clone(),
            snippets: all_snippets,
        };
        Ok(Some(page_document))
    }
}

pub async fn add_snippet_with_id(db: &SqlitePool, snippet: &str, document_id: i32) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO Snippet (snippet, document_id) VALUES ($1, $2) ON CONFLICT(snippet, document_id) DO NOTHING;")
    .bind(snippet)
    .bind(document_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn add_snippet(db: &SqlitePool, snippet: &str, document_name: &str) -> Result<()> {
    sqlx::query("INSERT INTO Document (document_name) VALUES ($1);")
        .bind(document_name)
        .execute(db)
        .await?;

    let document_row = sqlx::query_as::<_, Document>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name)
    .fetch_one(db)
    .await?;

    let document_id = document_row.document_id;
    sqlx::query("INSERT OR IGNORE INTO Snippet (snippet, document_id) VALUES ($1, $2) ON CONFLICT(snippet, document_id) DO NOTHING;")
    .bind(snippet)
    .bind(document_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn add_document(
    db: &SqlitePool,
    document_name: &str,
    snippet: &str,
    tfidf_terms: Vec<String>,
    rake_phrases: Vec<String>,
) -> Result<i32> {
    let document = sqlx::query_as::<_, Document>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name.trim())
    .fetch_optional(db)
    .await?;

    if let Some(document) = document {
        println!("Document with the corresponding title already exists");

        add_snippet_with_id(db, snippet, document.document_id).await?;
        update_tfidf_data(db, tfidf_terms, document.document_id).await?;
        update_rake_data(db, rake_phrases, document.document_id).await?;

        println!("Appended snippet to existing doc");

        return Ok(document.document_id);
    }

    sqlx::query(
        "INSERT INTO Document (document_name) VALUES ($1) ON CONFLICT(document_name) DO NOTHING;",
    )
    .bind(document_name.trim())
    .execute(db)
    .await?;

    let document_row = sqlx::query_as::<_, Document>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name)
    .fetch_one(db)
    .await?;

    let document_id = document_row.document_id;

    add_snippet_with_id(db, snippet, document_id).await?;
    update_tfidf_data(db, tfidf_terms, document_id).await?;
    update_rake_data(db, rake_phrases, document_id).await?;

    Ok(document_id)
}
