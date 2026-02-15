use crate::model::{DocStats, InvertedIndex};

pub fn compute_tf(term: &str, doc: &DocStats) -> f32 {
    let n = doc.total_words as f32;
    if n == 0.0 { return 0.0; }
    let f = *doc.tf.get(term).unwrap_or(&0) as f32;
    f / n
}

pub fn compute_idf(term: &str, index: &InvertedIndex) -> f32 {
    let n = index.docs.len() as f32;
    let m = index.docs.values().filter(|stats| stats.tf.contains_key(term)).count() as f32;
    (n / (1.0 + m)).log10()
}
