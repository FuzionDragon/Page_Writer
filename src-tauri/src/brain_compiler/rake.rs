use std::collections::{HashMap, HashSet};

use super::CorpusSnippets;

// the document isnt made up of single words but phrases
pub fn rake(document: Vec<String>) -> HashMap<String, f32> {
  let str_document: Vec<&str> = document.iter().map(|v| v.as_str()).collect();
  let mut words: Vec<&str> = Vec::new();

  for phrase in str_document.clone() {
    words.extend::<Vec<&str>>(phrase.split_whitespace().collect());
  }

  let unique_words: HashSet<&str> = HashSet::from_iter(words);

  let word_degrees = word_degrees(str_document.clone(), unique_words);
  let word_frequency = word_frequency(str_document.clone());
  let degree_scores = degree_scores(word_degrees, word_frequency);

  let mut scores = phrase_degree_scores(str_document, degree_scores.clone());
  scores.extend(degree_scores);

  scores
}

pub fn corpus_rake(corpus: CorpusSnippets) -> HashMap<String, HashMap<String, f32>> {
  let mut all_rake_scores: HashMap<String, HashMap<String, f32>> = HashMap::new();
  for (name, document) in corpus {
    all_rake_scores.insert(name, rake(document));
  }

  all_rake_scores
}

fn degree_scores(degree_of_words: HashMap<&str, f32>, word_frequency: HashMap<&str, f32>) -> HashMap<String, f32> {
  let mut degree_scores: HashMap<String, f32> = HashMap::new();

  for word in degree_of_words.clone().into_keys() {
    degree_scores.insert(word.to_string(), degree_of_words[word] / word_frequency[word]);
  }

  degree_scores
}

fn word_frequency(document: Vec<&str>) -> HashMap<&str, f32> {
  let mut frequencies: HashMap<&str, f32> = HashMap::new();
  let mut tokened_document: Vec<&str> = Vec::new();

  for phrase in document {
    tokened_document.extend::<Vec<&str>>(phrase.split_whitespace().collect());
  }

  for word in tokened_document {
    *frequencies.entry(word).or_default() += 1.;
  }

  frequencies
}

fn word_degrees<'a>(phrases: Vec<&'a str>, words: HashSet<&'a str>) -> HashMap<&'a str, f32> {
  let mut word_degrees: HashMap<&str, f32> = HashMap::new();

  for phrase in phrases {
    let phrase_words: Vec<&str> = phrase.split_whitespace().collect();
    for phrase_word in phrase_words.clone() {
      if words.contains(phrase_word) {
        *word_degrees.entry(phrase_word).or_default() += phrase_words.len() as f32;
      }
    }
  }

  word_degrees
}

fn phrase_degree_scores(phrases: Vec<&str>, degree_scores: HashMap<String, f32>) -> HashMap<String, f32> {
  let mut score: HashMap<String, f32> = HashMap::new();

  for phrase in phrases {
    let phrase_words = phrase.split_whitespace();
    for phrase_word in phrase_words {
      let degree_score = degree_scores.get(phrase_word).unwrap_or(&0.);
      *score.entry(phrase.to_string()).or_default() += degree_score;
    }
  }

  score
}
