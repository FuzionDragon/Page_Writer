use std::collections::{HashMap, HashSet};

// Scores are obtained from tfidf
pub fn cosine_similarity_tuple(
    scores_1: HashMap<String, f32>,
    scores_2: HashMap<String, f32>,
) -> f32 {
    let mut paired_scores: Vec<(f32, f32)> = Vec::new();
    let mut terms: HashSet<&String> = scores_1.keys().collect();
    terms.extend::<HashSet<&String>>(scores_2.keys().collect());

    for term in terms {
        paired_scores.push((
            scores_1.get(term).unwrap_or(&0.).to_owned(),
            scores_2.get(term).unwrap_or(&0.).to_owned(),
        ));
    }

    let dot_product = paired_scores.clone().iter().map(|v| v.0 * v.1).sum::<f32>();

    let magnitude_a = paired_scores
        .clone()
        .iter()
        .map(|v| v.0.powi(2))
        .sum::<f32>()
        .sqrt();

    let magnitude_b = paired_scores
        .iter()
        .map(|v| v.1.powi(2))
        .sum::<f32>()
        .sqrt();

    if magnitude_a == 0. || magnitude_b == 0. {
        0.
    } else {
        dot_product / (magnitude_a * magnitude_b)
    }
}

// weights base on RAKE algorithm scores
pub fn weighted_jaccard_similarity(
    document_1: Vec<String>,
    document_2: Vec<String>,
    document_1_scores: HashMap<String, f32>,
    document_2_scores: HashMap<String, f32>,
) -> f32 {
    let scores_1: HashMap<String, f32> = HashMap::from_iter(document_1_scores);
    let scores_2: HashMap<String, f32> = HashMap::from_iter(document_2_scores);
    let mut all_words: HashSet<&str> = document_1.iter().map(|v| v.as_str()).collect();
    all_words.extend::<HashSet<&str>>(document_2.iter().map(|v| v.as_str()).collect());

    let minimum = all_words
        .clone()
        .into_iter()
        .map(|k| {
            scores_1
                .get(k)
                .unwrap_or(&0.)
                .min(*scores_2.get(k).unwrap_or(&0.))
        })
        .sum::<f32>();

    let maximum = all_words
        .into_iter()
        .map(|k| {
            scores_1
                .get(k)
                .unwrap_or(&0.)
                .max(*scores_2.get(k).unwrap_or(&0.))
        })
        .sum::<f32>();

    minimum / maximum
}
