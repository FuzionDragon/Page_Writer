use anyhow::Ok;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use stop_words::{get, LANGUAGE};

pub mod preprocess;
pub mod rake;
pub mod similarity;
pub mod sqlite_interface;
pub mod tf_idf;

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

const COSINE_WEIGHT: f32 = 0.6;
const LATEST_BIAS: f32 = 0.25;
const THESHOLD: f32 = 0.4;

pub async fn submit_snippet(
    snippet: &str,
    title: Option<&str>,
    db: &SqlitePool,
) -> Result<Option<i32>, anyhow::Error> {
    if snippet.is_empty() {
        println!("Snippet is empty");

        return Ok(None);
    };

    let stop_words = get(LANGUAGE::English);

    let input_tfidf_data = preprocess::tfidf_preprocess(snippet, stop_words.clone());
    let input_rake_data = preprocess::rake_preprocess(snippet, stop_words.clone());

    let corpus_tfidf_data = sqlite_interface::load_tfidf_data(db).await?;
    let corpus_rake_data = sqlite_interface::load_rake_data(db).await?;

    if let Some(title) = title {
        let document_id = sqlite_interface::add_document(
            db,
            title.trim(),
            snippet,
            input_tfidf_data.clone(),
            input_rake_data.clone(),
        )
        .await?;
        sqlite_interface::set_latest_document(db, document_id).await?;

        Ok(Some(document_id))
    } else {
        let first_line = snippet.lines().collect::<Vec<&str>>()[0];
        let marked_document = sqlite_interface::fetch_marked_document(db).await?;
        let latest_document = sqlite_interface::fetch_latest_document(db).await?;

        if let Some(marked_document) = marked_document {
            let document_id = sqlite_interface::add_document(
                db,
                &marked_document.document_name,
                snippet,
                input_tfidf_data.clone(),
                input_rake_data.clone(),
            )
            .await?;

            return Ok(Some(document_id));
        }

        let scores = combined_similarity_scores(
            input_tfidf_data.clone(),
            input_rake_data.clone(),
            corpus_tfidf_data,
            corpus_rake_data,
            COSINE_WEIGHT,
            LATEST_BIAS,
            latest_document,
        );

        if scores.is_empty() {
            let document_id = sqlite_interface::add_document(
                db,
                first_line.trim(),
                snippet,
                input_tfidf_data.clone(),
                input_rake_data.clone(),
            )
            .await?;
            sqlite_interface::set_latest_document(db, document_id).await?;

            return Ok(Some(document_id));
        }

        if scores[0].1 >= THESHOLD {
            println!(
                "{} is the chosen document with a score of {}",
                scores[0].0, scores[0].1
            );

            let document_id = sqlite_interface::add_document(
                db,
                &scores[0].0,
                snippet,
                input_tfidf_data.clone(),
                input_rake_data.clone(),
            )
            .await?;
            sqlite_interface::set_latest_document(db, document_id).await?;

            Ok(Some(document_id))
        } else {
            println!(
                "{} doesn't meet the threshold with a score of {}",
                scores[0].0, scores[0].1
            );
            println!("Creating new document");
            let document_id = sqlite_interface::add_document(
                db,
                first_line.trim(),
                snippet,
                input_tfidf_data,
                input_rake_data,
            )
            .await?;
            sqlite_interface::set_latest_document(db, document_id).await?;

            Ok(Some(document_id))
        }
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

    let stop_words = get(LANGUAGE::English);

    let input_tfidf_data = preprocess::tfidf_preprocess(snippet, stop_words.clone());
    let input_rake_data = preprocess::rake_preprocess(snippet, stop_words.clone());

    sqlite_interface::update_snippet(db, snippet_id, snippet, input_tfidf_data, input_rake_data)
        .await?;

    Ok(())
}

fn combined_similarity_scores(
    input_tfidf_data: Vec<String>,
    input_rake_data: Vec<String>,
    corpus_tfidf_data: CorpusSnippets,
    corpus_rake_data: CorpusSnippets,
    cosine_weight: f32,
    latest_bias: f32,
    latest_document: Option<PageDocument>,
) -> Vec<(String, f32)> {
    let corpus_tfidf_scores = tf_idf::corpus_tf_idf_hash(corpus_tfidf_data.clone());
    let corpus_rake_scores = rake::corpus_rake(corpus_rake_data.clone());

    let tf_idf_input_score = tf_idf::tf_idf_hash(input_tfidf_data, corpus_tfidf_data);
    let rake_input_score = rake::rake(input_rake_data.clone());

    let documents_1: HashSet<&str> = corpus_tfidf_scores.keys().map(|k| k.as_str()).collect();
    let documents_2: HashSet<&str> = corpus_rake_scores.keys().map(|k| k.as_str()).collect();
    let all_documents: HashSet<&str> = documents_1
        .union(&documents_2)
        .map(|v| v.to_owned())
        .collect();

    let mut combined_scores: HashMap<String, f32> = HashMap::new();

    for document in all_documents {
        let mut empty: HashMap<String, f32> = HashMap::new();
        empty.insert(document.to_string(), 0.);

        let corpus_tfidf_score = corpus_tfidf_scores.get(document).unwrap_or(&empty).clone();
        let cosine_similarity_score =
            similarity::cosine_similarity_tuple(tf_idf_input_score.clone(), corpus_tfidf_score)
                * cosine_weight;

        let corpus_rake_score = corpus_rake_scores.get(document).unwrap_or(&empty).clone();
        let weighted_jaccard_similarity_score = similarity::weighted_jaccard_similarity(
            input_rake_data.clone(),
            corpus_rake_data
                .get(document)
                .unwrap_or(&Vec::new())
                .clone(),
            rake_input_score.clone(),
            corpus_rake_score,
        ) * (1. - cosine_weight);

        match &latest_document {
            Some(latest_document) => {
                if latest_document.document_name == document {
                    combined_scores.insert(
                        document.to_string(),
                        cosine_similarity_score + weighted_jaccard_similarity_score + latest_bias,
                    );
                }
            }

            None => {
                combined_scores.insert(
                    document.to_string(),
                    cosine_similarity_score + weighted_jaccard_similarity_score,
                );
            }
        }
    }

    let mut sorted_scores: Vec<(String, f32)> = combined_scores.into_iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    sorted_scores
}
