use anyhow::Ok;
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

const COSINE_WEIGHT: f32 = 0.6;
const THESHOLD: f32 = 0.4;

pub async fn submit_snippet(snippet: &str, db: &SqlitePool) -> Result<(), anyhow::Error> {
    let first_entry =
        sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Document'")
            .fetch_all(db)
            .await?
            .is_empty();

    if snippet.is_empty() {
      println!("Snippet is empty");
      return Ok(())
    };

    let first_line = snippet.lines().collect::<Vec<&str>>()[0];
    let title: Option<&str> = first_line.strip_prefix('#');

    let stop_words = get(LANGUAGE::English);

    let input_tfidf_data = preprocess::tfidf_preprocess(snippet, stop_words.clone());
    let input_rake_data = preprocess::rake_preprocess(snippet, stop_words.clone());

    if first_entry {
        sqlite_interface::init(db).await?;

        if let Some(title) = title {
            let document_exists = !sqlite_interface::add_document(db, title.trim(), snippet, input_tfidf_data, input_rake_data).await?;
            if document_exists {
                println!("Document with that title already exists");
            }
        } else {
            let first_document = "first document";
            sqlite_interface::add_document(db, first_document, snippet, input_tfidf_data, input_rake_data).await?;
        }
    } else {
        let corpus_tfidf_data = sqlite_interface::load_tfidf_data(db).await?;
        let corpus_rake_data = sqlite_interface::load_rake_data(db).await?;

        if let Some(title) = title {
            let document_exists = !sqlite_interface::add_document(db, title, snippet, input_tfidf_data.clone(), input_rake_data.clone()).await?;
            if document_exists {
                let mut snippet_vec = snippet.split("\n").collect::<Vec<&str>>();
                snippet_vec.remove(0);
                if snippet_vec.is_empty() {
                  println!("Snippet is empty");
                  return Ok(())
                };
                let new_snippet = snippet_vec.join("\n");

                sqlite_interface::add_document(db, title, &new_snippet, input_tfidf_data, input_rake_data).await?;
                println!("Document title found, pushed snippet to existing doc");
            } else {
                println!("Document title not found, creating new doc");
            }
        } else {
            let scores = combined_similarity_scores(
                input_tfidf_data.clone(),
                input_rake_data.clone(),
                corpus_tfidf_data,
                corpus_rake_data,
                COSINE_WEIGHT,
            );

            if scores[0].1 >= THESHOLD {
                println!(
                    "{} is the chosen document with a score of {}",
                    scores[0].0, scores[0].1
                );

                sqlite_interface::add_snippet(db, snippet, &scores[0].0).await?;
                sqlite_interface::update_tfidf_data(db, input_tfidf_data, &scores[0].0).await?;
                sqlite_interface::update_rake_data(db, input_rake_data, &scores[0].0).await?;
            } else {
                println!(
                    "{} doesn't meet the threshold with a score of {}",
                    scores[0].0, scores[0].1
                );
                println!("Creating new document");
                sqlite_interface::add_document(db, first_line.trim(), snippet, input_tfidf_data, input_rake_data).await?;
            }
        }
    }

    Ok(())
}

fn combined_similarity_scores(
    input_tfidf_data: Vec<String>,
    input_rake_data: Vec<String>,
    corpus_tfidf_data: CorpusSnippets,
    corpus_rake_data: CorpusSnippets,
    cosine_weight: f32,
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
        let cosine_similarity_score = similarity::cosine_similarity_tuple(
            tf_idf_input_score.clone(),
            corpus_tfidf_scores[document].clone(),
        ) * cosine_weight;

        let weighted_jaccard_similarity_score = similarity::weighted_jaccard_similarity(
            input_rake_data.clone(),
            corpus_rake_data[document].clone(),
            rake_input_score.clone(),
            corpus_rake_scores[document].clone(),
        ) * (1. - cosine_weight);

        combined_scores.insert(
            document.to_string(),
            cosine_similarity_score + weighted_jaccard_similarity_score,
        );
    }

    let mut sorted_scores: Vec<(String, f32)> = combined_scores.into_iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    sorted_scores
}
