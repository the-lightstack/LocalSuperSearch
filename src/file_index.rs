use std::collections::HashMap;

use std::fs::read_to_string;
use std::path::PathBuf;

use keyword_extraction::text_rank::{TextRank, TextRankParams};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use stop_words::{get, LANGUAGE};

use crate::crawl::{get_file_extension, FileType};

const AMOUNT_KEYWORDS: usize = 20;
const LANG_ANALYSIS_FIRST_CHUNK: usize = 100;

// static STOP_WORDS: Vec<std::string::String> = get(LANGUAGE::English);

#[derive(Debug, Clone)]
struct LanguageConversionFailError;

fn stopword_lang_from_lingua_lang(
    lang: lingua::Language,
) -> Result<LANGUAGE, LanguageConversionFailError> {
    // TODO: macro to use all languages here
    match lang {
        lingua::Language::German => Ok(LANGUAGE::German),
        lingua::Language::English => Ok(LANGUAGE::English),
        lingua::Language::Spanish => Ok(LANGUAGE::Spanish),
        lingua::Language::French => Ok(LANGUAGE::French),
        lingua::Language::Polish => Ok(LANGUAGE::Polish),
        lingua::Language::Hebrew => Ok(LANGUAGE::Hebrew),
        lingua::Language::Irish => Ok(LANGUAGE::Irish),
        lingua::Language::Catalan => Ok(LANGUAGE::Catalan),
        lingua::Language::Croatian => Ok(LANGUAGE::Croatian),
        lingua::Language::Chinese => Ok(LANGUAGE::Chinese),
        lingua::Language::Czech => Ok(LANGUAGE::Czech),
        lingua::Language::Hindi => Ok(LANGUAGE::Hindi),
        lingua::Language::Danish => Ok(LANGUAGE::Danish),
        lingua::Language::Dutch => Ok(LANGUAGE::Dutch),
        lingua::Language::Korean => Ok(LANGUAGE::Korean),

        _ => Err(LanguageConversionFailError),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Keyword {
    pub score: f32,
    pub word: String,
}

// used very rarely, so clone should be fine
impl From<&(String, f32)> for Keyword {
    fn from(value: &(String, f32)) -> Self {
        Self {
            score: value.1,
            word: value.0.clone(),
        }
    }
}

// We want to do caching for different language's stop_words
pub struct Indexer {
    _stop_words_cache: HashMap<lingua::Language, Vec<String>>,
    _language_detector: LanguageDetector,
}

#[derive(Debug, PartialEq)]
pub struct CannotExtractKeywordsError {}

impl Indexer {
    pub fn init() -> Self {
        let supported_languages = vec![
            Language::German,
            Language::English,
            Language::Spanish,
            Language::French,
            Language::Polish,
            Language::Hebrew,
            Language::Irish,
            Language::Catalan,
            Language::Croatian,
            Language::Chinese,
            Language::Czech,
            Language::Hindi,
            Language::Danish,
            Language::Dutch,
            Language::Korean,
        ];
        let __language_detector =
            LanguageDetectorBuilder::from_languages(&supported_languages).build();

        Self {
            _stop_words_cache: HashMap::new(),
            _language_detector: __language_detector,
        }
    }

    /// Extract raw contents based on file type, then find out language for stop words and finally
    /// find key words using TextRank
    pub fn get_keywords_from_path(
        &mut self,
        file_path: &PathBuf,
    ) -> Result<Vec<Keyword>, CannotExtractKeywordsError> {
        // Identifying file type
        let ext = get_file_extension(file_path.file_name().unwrap().to_str().unwrap());
        let filetype = FileType::get(ext);

        let content: Option<String> = match filetype {
            // Plain text
            FileType::Markdown
            | FileType::Config
            | FileType::Web
            | FileType::Python
            | FileType::JavaScript
            | FileType::Rust
            | FileType::CSOURCE
            | FileType::Plain
            | FileType::Excel => {
                let c_res = read_to_string(&file_path);
                match c_res {
                    Ok(c) => Some(c),
                    Err(_) => None,
                }
            }

            // Pdf Parsing
            FileType::PDF => {
                let bytes_res = std::fs::read(&file_path);
                match bytes_res {
                    Ok(bytes) => {
                        let out = pdf_extract::extract_text_from_mem(&bytes);

                        match out {
                            Ok(cont) => Some(cont),
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            }

            FileType::LibreWriter => todo!("Add parser for Word style documents"),
            FileType::Presentation => todo!("Add parser for presentation types"),
            FileType::Unknown => None,
        };

        let content = content.ok_or(CannotExtractKeywordsError {})?;

        // Test for language

        // If not a big file, take all and analyse
        let detection_text_snippet = if content.len() < LANG_ANALYSIS_FIRST_CHUNK {
            &content
        } else {
            // TODO: check when this fails
            &content
                .get(0..LANG_ANALYSIS_FIRST_CHUNK)
                .unwrap()
                .to_string()
        };


        let text_language = self
            ._language_detector
            .detect_language_of(detection_text_snippet)
            .ok_or(CannotExtractKeywordsError {})?;

        println!("Language is: {:?}", text_language);

        // Stop words
        let keywords = self
            .extract_keywords(&content, text_language)
            .expect("extract keywords, should not fail");

         Ok(keywords)
    }

    fn extract_keywords(
        &mut self,
        raw_text: &str,
        language: Language,
    ) -> Result<Vec<Keyword>, LanguageConversionFailError> {
        let stop_words = {
            match self._stop_words_cache.get(&language) {
                Some(l) => Ok(l.clone()),
                None => {
                    // If cache lookup fails
                    let sw_lang = stopword_lang_from_lingua_lang(language)?;
                    let internal_stop_words = get(sw_lang);
                    self._stop_words_cache
                        .insert(language, internal_stop_words.clone());

                    Ok(internal_stop_words)
                }
            }?
        };

        let text_rank = TextRank::new(TextRankParams::WithDefaults(raw_text, &stop_words));
        Ok(text_rank
            .get_ranked_phrase_scores(AMOUNT_KEYWORDS)
            .iter()
            .map(|i| Keyword::from(i))
            .collect())
    }
}

// TODO: make this read in files *Buffered* and turn to keywords in chunks.
// Otherwise we will quickly run out of memory, especially when running in more threads
// fn extract_keywords(file_path: &PathBuf)->Result<String>{
//     // check how expensive replacing is
//    let byte_contents =  fs::read(file_path)?;

//     let text_rank = TextRank::new(TextRankParams::WithDefaults(&contents, &STOP_WORDS));
//     let ranked_keywords = text_rank.get_ranked_words(AMOUNT_KEYWORDS);

//     // println!("KWS: {:?}",ranked_keywords);

// }
