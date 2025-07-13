use std::collections::{HashMap, HashSet};

use super::CorpusSnippets;

pub fn tf_idf(term: &str, document: Vec<&str>, corpus: CorpusSnippets) -> f32 {
  let str_corpus: Vec<Vec<&str>> = corpus
    .values()
    .map(|v| v.iter()
      .map(|v| v.as_str())
      .collect())
    .collect();
  tf(term, document) * idf(term, str_corpus)
}

pub fn corpus_tf_idf_hash(corpus: CorpusSnippets) -> HashMap<String, HashMap<String, f32>> {
  let mut hashes: HashMap<String, HashMap<String, f32>> = HashMap::new();
  let mut all_terms: HashSet<&str> = HashSet::new();

  let corpus_documents: Vec<Vec<&str>> = corpus.values().map(|v| v.iter().map(|v| v.as_str()).collect()).collect();
  for document in corpus_documents {
    all_terms.extend(document);
  }

  for (name, document) in corpus.clone() {
    let str_document: Vec<&str> = document.iter().map(|v| v.as_str()).collect();
    for term in all_terms.clone() {
      *hashes.entry(name.to_string())
        .or_default()
        .entry(term.to_string())
        .or_default() 
        += tf_idf(term, str_document.clone(), corpus.clone());
    }
  }

  hashes
}

pub fn tf_idf_hash(document: Vec<String>, corpus: CorpusSnippets) -> HashMap<String, f32> {
  let str_document: Vec<&str> = document.iter().map(|v| v.as_str()).collect();
  let mut scores: HashMap<String, f32> = HashMap::new();
  let mut all_terms: HashSet<&str> = HashSet::new();

  let corpus_documents: Vec<Vec<&str>> = corpus.values().map(|v| v.iter().map(|v| v.as_str()).collect()).collect();
  for document in corpus_documents {
    all_terms.extend(document);
  }

  for term in all_terms {
    scores.insert(term.to_string(), tf_idf(term, str_document.clone(), corpus.clone()));
  }

  scores
}

fn tf(search_term: &str, document: Vec<&str>) -> f32 {
  let mut search_term_count = 0.;
  let mut all_term_count = 0.;

  for term in document {
    if term == search_term {
      search_term_count += 1.;
    }

    all_term_count += 1.;
  }

  search_term_count / all_term_count
}

fn idf(term: &str, corpus: Vec<Vec<&str>>) -> f32 {
  let mut count: f32 = 0.;
  let mut total_documents: f32 = 0.;

  for document in corpus {
    if document.contains(&term) {
      count += 1.;
    }

    total_documents += 1.;
  };

  (total_documents / count).ln()
}
