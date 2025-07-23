use human_regex::{one_or_more, punctuation};
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::HashMap;

use super::CorpusSnippets;
type Corpus = HashMap<String, String>;

pub fn corpus_tfidf_preprocess(corpus: Corpus, stop_words: Vec<String>) -> CorpusSnippets {
    let mut processed: CorpusSnippets = HashMap::new();

    for (name, document) in corpus {
        processed.insert(
            name,
            tfidf_preprocess(document.as_str(), stop_words.clone()),
        );
    }

    processed
}

pub fn corpus_rake_preprocess(corpus: Corpus, stop_words: Vec<String>) -> CorpusSnippets {
    let mut processed: CorpusSnippets = HashMap::new();

    for (name, document) in corpus {
        processed.insert(name, rake_preprocess(document.as_str(), stop_words.clone()));
    }

    processed
}

pub fn rake_preprocess(document: &str, stop_words: Vec<String>) -> Vec<String> {
    let en_stemmer = Stemmer::create(Algorithm::English);
    let lowercase_text = document.to_string().to_ascii_lowercase();
    let punctuation_regex = one_or_more(punctuation());
    let no_punctuation_text = punctuation_regex
        .to_regex()
        .replace_all(&lowercase_text, "");

    let clean_text: Vec<String> = no_punctuation_text
        .split_whitespace()
        .map(|word| en_stemmer.stem(word).to_string())
        .collect();

    let mut phrases: Vec<String> = Vec::with_capacity(document.len());
    let mut phrase: Vec<String> = Vec::new();

    for word in clean_text {
        if stop_words.contains(&word) && !phrase.is_empty() {
            phrases.push(phrase.clone().join(" "));
            phrase.clear();
        } else {
            phrase.push(word);
        }
    }
    if !phrase.is_empty() {
        phrases.push(phrase.join(" "));
    }

    phrases
        .into_iter()
        .filter(|word| !stop_words.contains(&word.to_string()))
        .collect::<Vec<String>>()
}

pub fn tfidf_preprocess(document: &str, stop_words: Vec<String>) -> Vec<String> {
    let en_stemmer = Stemmer::create(Algorithm::English);
    let lowercase_text = document.to_string().to_ascii_lowercase();
    let punctuation_regex = one_or_more(punctuation());
    let no_punctuation_text = punctuation_regex
        .to_regex()
        .replace_all(&lowercase_text, "");

    let clean_text: Vec<String> = no_punctuation_text
        .split_whitespace()
        .filter(|word| !stop_words.contains(&word.to_string()))
        .map(|word| en_stemmer.stem(word).to_string())
        .collect();

    clean_text
}
