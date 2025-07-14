use std::collections::HashMap;

use anyhow::{Ok, Result};
use sqlx::{FromRow, SqlitePool};

use super::{Corpus, CorpusSnippets};

type Document = HashMap<String, String>;

#[derive(Debug, FromRow, Clone)]
pub struct Snippet {
    snippet: String,
    document: String,
}

#[derive(Debug, FromRow, Clone)]
pub struct SnippetRow {
    snippet_id: i32,
    snippet: String,
    document_id: i32,
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
struct DocumentRow {
    document_id: i32,
    document_name: String,
}

pub async fn init(db: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
    CREATE TABLE IF NOT EXISTS Document (
      document_id INTEGER PRIMARY KEY AUTOINCREMENT,
      document_name TEXT UNIQUE
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
    SELECT Document.document_name, snippet FROM Snippet
    LEFT JOIN Document ON Snippet.document_id == Document.document_id;
  "#,
    )
    .fetch_all(db)
    .await?;

    let mut corpus_snippets: CorpusSnippets = HashMap::new();
    for snippet in snippets {
        corpus_snippets
            .entry(snippet.document)
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

pub async fn update_tfidf_data(db: &SqlitePool, terms: Vec<String>, document: &str) -> Result<()> {
    let document_row =
        sqlx::query_as::<_, DocumentRow>("SELECT * FROM Document WHERE document_name = $1;")
            .bind(document)
            .fetch_one(db)
            .await?;

    let snippet_row =
        sqlx::query_as::<_, SnippetRow>("SELECT * FROM Snippet WHERE document_id = $1;")
            .bind(document_row.document_id)
            .fetch_one(db)
            .await?;

    for term in terms {
        sqlx::query("INSERT OR IGNORE INTO TFIDF_Term (term, snippet_id) VALUES ($1, $2) ON CONFLICT(term, snippet_id) DO NOTHING;")
      .bind(term)
      .bind(snippet_row.snippet_id)
      .execute(db)
      .await?;
    }

    Ok(())
}

pub async fn update_rake_data(db: &SqlitePool, phrases: Vec<String>, document: &str) -> Result<()> {
    let document_row =
        sqlx::query_as::<_, DocumentRow>("SELECT * FROM Document WHERE document_name = $1;")
            .bind(document)
            .fetch_one(db)
            .await?;

    let snippet_row =
        sqlx::query_as::<_, SnippetRow>("SELECT * FROM Snippet WHERE document_id = $1;")
            .bind(document_row.document_id)
            .fetch_one(db)
            .await?;

    for phrase in phrases {
        sqlx::query("INSERT OR IGNORE INTO RAKE_Phrase (phrase, snippet_id) VALUES ($1, $2) ON CONFLICT(phrase, snippet_id) DO NOTHING;")
      .bind(phrase)
      .bind(snippet_row.snippet_id)
      .execute(db)
      .await?;
    }

    Ok(())
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

    let document_row = sqlx::query_as::<_, DocumentRow>(
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
) -> Result<bool> {
    let document_exists = !sqlx::query_as::<_, DocumentRow>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name)
    .fetch_all(db)
    .await?
    .is_empty();

    if document_exists {
        println!("Document with the corresponding title already exists");
        return Ok(false);
    }

    sqlx::query("INSERT INTO Document (document_name) VALUES ($1) ON CONFLICT(document_name) DO NOTHING;")
    .bind(document_name)
    .execute(db)
    .await?;

    let document_row = sqlx::query_as::<_, DocumentRow>(
        "SELECT document_id, document_name FROM Document WHERE document_name = $1;",
    )
    .bind(document_name)
    .fetch_one(db)
    .await?;

    let document_id = document_row.document_id;

    add_snippet_with_id(db, snippet, document_id).await?;
    update_tfidf_data(db, tfidf_terms, document_name).await?;
    update_rake_data(db, rake_phrases, document_name).await?;

    Ok(true)
}
