use std::collections::HashSet;
use std::sync::OnceLock;

pub fn is_stopword(token: &str) -> bool {
    static STOPWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    let stopwords = STOPWORDS.get_or_init(|| {
        [
            "A", "AN", "AND", "ARE", "AS", "AT", "BE", "BUT", "BY",
            "FOR", "IF", "IN", "INTO", "IS", "IT", "NO", "NOT", "OF",
            "ON", "OR", "SUCH", "THAT", "THE", "THEIR", "THEN", "THERE",
            "THESE", "THEY", "THIS", "TO", "WAS", "WILL", "WITH"
        ].iter().cloned().collect()
    });
    stopwords.contains(token)
}
