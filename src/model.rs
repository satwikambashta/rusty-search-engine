use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DocStats {
    pub tf: HashMap<String, usize>,
    pub total_words: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct InvertedIndex {
    pub docs: HashMap<PathBuf, DocStats>,
}
