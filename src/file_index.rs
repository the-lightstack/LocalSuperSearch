use std::path::PathBuf;
use std::fs;
use keyword_extraction::{rake::{Rake, RakeParams}, text_rank::{TextRank, TextRankParams}};
use stop_words::{get,LANGUAGE};

const AMOUNT_KEYWORDS:usize = 20;

// TODO: make this read in files *Buffered* and turn to keywords in chunks.
// Otherwise we will quickly run out of memory, especially when running in more threads
pub fn extract_keywords(file_path: &PathBuf)->String{
    let contents = fs::read_to_string(file_path).expect("Should be able to read file");

    let stop_words = get(LANGUAGE::English);
    let punctuation: Vec<String> =[
        ".", ",", ":", ";", "!", "?", "(", ")", "[", "]", "{", "}", "\"", "'",
    ].iter().map(|s| s.to_string()).collect();

    let text_rank = TextRank::new(TextRankParams::WithDefaults(&contents, &stop_words));
    let ranked_keywords = text_rank.get_ranked_words(AMOUNT_KEYWORDS);

    println!("KWS: {:?}",ranked_keywords);

    ranked_keywords.join("|")
}