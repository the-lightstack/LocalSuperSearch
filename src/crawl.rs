use core::{panic};
use rusqlite::types::FromSql;
use rusqlite::Connection;
use std::convert::From;
use std::fs::{self};
use std::{path::PathBuf, time::UNIX_EPOCH};

extern crate queues;
use queues::*;

use crate::file_index::{Indexer, Keyword};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Markdown = 1,
    Config = 2,
    Web = 3,
    Python = 4,
    JavaScript = 5,
    Rust = 6,
    CSOURCE = 7,
    Presentation = 8,
    PDF = 9,
    LibreWriter = 10,
    Excel = 11,
    Plain = 12,

    Unknown = 13,
}
impl From<i32> for FileType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Markdown,
            2 => Self::Config,
            3 => Self::Web,
            4 => Self::Python,
            5 => Self::JavaScript,
            6 => Self::Rust,
            7 => Self::CSOURCE,
            8 => Self::Presentation,
            9 => Self::PDF,
            10 => Self::LibreWriter,
            11 => Self::Excel,
            12 => Self::Plain,
            13 => Self::Unknown,
            _ => {
                panic!("Encountered Invalid File Type in Database")
            }
        }
    }
}

impl FromSql for FileType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(FileType::from(value.as_i64().unwrap() as usize))
    }
}

impl FileType {
    pub fn get(file_ext: Option<String>) -> Self {
        match file_ext {
            Some(ext) => match ext.as_str() {
                "md" => Self::Markdown,
                "yml" => Self::Config,
                "yaml" => Self::Config,
                "json" => Self::Config,
                "config" => Self::Config,
                "toml" => Self::Config,
                "xml" => Self::Config,
                "html" => Self::Web,
                "htmx" => Self::Web,
                "css" => Self::Web,
                "py" => Self::Python,
                "js" => Self::JavaScript,
                "ts" => Self::JavaScript,
                "rs" => Self::Rust,
                "c" => Self::CSOURCE,
                "cpp" => Self::CSOURCE,
                "h" => Self::CSOURCE,
                "hpp" => Self::CSOURCE,
                "ppt" => Self::Presentation,
                "pptx" => Self::Presentation,
                "pps" => Self::Presentation,
                "ppsx" => Self::Presentation,
                "pot" => Self::Presentation,
                "potx" => Self::Presentation,
                "odp" => Self::Presentation,
                "odkey" => Self::Presentation,
                "doc" => Self::LibreWriter,
                "docx" => Self::LibreWriter,
                "dot" => Self::LibreWriter,
                "dotx" => Self::LibreWriter,
                "odt" => Self::LibreWriter,
                "ott" => Self::LibreWriter,
                "pages" => Self::LibreWriter,
                "rtf" => Self::LibreWriter,
                "txt" => Self::Plain,
                "pdf" => Self::PDF,
                "csv" => Self::Excel,

                _ => Self::Unknown,
            },
            None => Self::Unknown,
        }
    }
}

impl From<usize> for FileType {
    fn from(value: usize) -> Self {
        FileType::from(value as i32)
    }
}

#[derive(Debug)]
struct IndexEntry {
    filename: String,

    filetype: FileType,
    filepath: PathBuf,
    keywords: Vec<Keyword>,
    last_modified_timestamp: u128,
}

#[derive(Debug)]
struct SearchResult {
    filename: String,
    filepath: String,
    filetype: FileType,
    matching_keyword: String,
    match_score: f32,
}

pub struct CrawlDatabase {
    _conn: Connection,
    _search_queue: Queue<PathBuf>,
    _indexer: Indexer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_init_and_insert() {
        let mut cdb = CrawlDatabase::init(":memory:");
        let ie = IndexEntry {
            filename: String::from("test_file_name.txt"),
            filetype: FileType::Config,
            filepath: PathBuf::from("/test/test_file_name.txt"),
            keywords: vec![],
            last_modified_timestamp: 92738728374,
        };

        cdb.store_new_index(&ie);
    }

    #[test]
    fn get_file_extension_test() {
        assert_eq!(None, get_file_extension("nothing"));
        assert_eq!(None, get_file_extension("nothing."));
        assert_eq!(Some("js"), get_file_extension("file.js").as_deref());
        assert_eq!(
            Some("config"),
            get_file_extension("big_filename.config").as_deref()
        );
    }

    #[test]
    fn get_enum_from_number() {
        let n = 4;
        let e = FileType::from(n);
        assert_eq!(e, FileType::Python);
    }

    #[test]
    fn filetype_from_sql(){
        // SQL

    }

}

pub fn get_file_extension(name: &str) -> Option<String> {
    if !name.contains(".") {
        None
    } else {
        let ext = name.split(".").last()?;
        if ext.is_empty() {
            None
        } else {
            Some(name.split(".").last()?.to_lowercase())
        }
    }
}

#[derive(Debug,PartialEq)]
enum FileCrawlStatus{
    FileNotChanged,
    FileChanged,
    FirstFileIndex
}

const INDEXABLE_FILE_EXTENSIONS: &[&str] = &[
    "md", "ppt", "pptx", "pps", "ppsx", "pot", "potx", "odp", "odkey",
    "doc", "docx", "dot", "dotx", "odt", "ott", "pages", "rtf", "txt", "pdf",
];

const EXCLUDE_DIRS: &[&str] = &[".git", ".yarn", ".var", "venv", "__pycache__"];
impl CrawlDatabase {
    pub fn init(path: &str) -> Self {
        let conn = Connection::open(path).unwrap();

        // We want to use foreign keys for relationships
        conn.execute("PRAGMA foreign_keys = ON", ()).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS search_index (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

            filename STRING NOT NULL,
            filetype INT NOT NULL,
            filepath STRING UNIQUE NOT NULL,
            last_modified_timestamp TIMESTAMP
        ) ",
            (),
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS keywords (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            si_id REFERENCES search_index(id) ON DELETE CASCADE,
            word STRING NOT NULL,
            score FLOAT NOT NULL

        ) ",
            (),
        )
        .unwrap();

        Self {
            _conn: conn,
            _search_queue: queue![],
            _indexer: Indexer::init(),
        }
    }

    fn format_results(&self, sr: SearchResult) {
        println!("{} [{} -> {}]", sr.filename, sr.match_score, sr.filepath);
    }

    pub fn search_keyword(&self, keyword: &str) {


        let mut stmt = self._conn.prepare("SELECT S.filename,S.filepath, S.filetype,K.word,K.score FROM keywords K INNER JOIN search_index S ON K.si_id=S.id
                                WHERE K.word LIKE :search ORDER BY K.score DESC LIMIT 20").unwrap();


        let params = &[(":search",&format!("%{}%", keyword))];
        let result_iter = stmt
            .query_map(params, |row| {
                Ok(SearchResult {
                    filename: row.get(0)?,
                    filepath: row.get(1)?,
                    filetype: row.get(2)?,
                    matching_keyword: row.get(3)?,
                    match_score: row.get(4)?,
                })
            })
            .unwrap();

        for search_result in result_iter {
            match search_result {
                Ok(sr) => self.format_results(sr),
                Err(_) => {
                    panic!("check here")
                }
            }
        }


    }

    fn store_new_index(&mut self, ie: &IndexEntry) {
        self._conn.execute("INSERT INTO search_index (filename, filetype, filepath, last_modified_timestamp) VALUES (?1,?2,?3,?4)", (&ie.filename,ie.filetype as i64,ie.filepath.to_str(),ie.last_modified_timestamp as u64)).unwrap();

        let last_rowid = self._conn.last_insert_rowid();

        for kw in &ie.keywords {
            self._conn
                .execute(
                    "INSERT INTO keywords (si_id, word, score) VALUES (?1, ?2, ?3)",
                    (last_rowid, &kw.word, kw.score),
                )
                .unwrap();
        };
    }

    fn update_index(&mut self, ie: &IndexEntry) {
        self._conn.execute("UPDATE search_index SET filename=?1, filetype=?2, last_modified_timestamp=?3 WHERE filepath=?4",
    (&ie.filename,
            ie.filetype as i64,
            ie.last_modified_timestamp as u64,
            ie.filepath.to_str())).unwrap();

        let last_rowid = self._conn.last_insert_rowid();

        // Delete all old Keywords
        self._conn.execute("DELETE FROM keywords WHERE si_id=?1",(last_rowid,)).unwrap();

        // And insert the new ones
        for kw in &ie.keywords {
            self._conn
                .execute(
                    "INSERT INTO keywords (si_id, word, score) VALUES (?1, ?2, ?3)",
                    (last_rowid, &kw.word, kw.score),
                )
                .unwrap();
        };
    }

    fn index_file(&mut self, file_path: &PathBuf) {
        let meta = fs::metadata(file_path).expect("Expected to be able to read MetaData on file");

        // Check if modified time has changed
        let last_modified = meta.modified().expect("Cannot read last modified");

        let file_crawl_status = self.check_needs_crawl(file_path.to_str().unwrap(), last_modified.duration_since(UNIX_EPOCH).unwrap().as_millis());
        if file_crawl_status == FileCrawlStatus::FileNotChanged  {return;}



        let keywords = match self
            ._indexer
            .get_keywords_from_path(file_path){
                Ok(k) =>k,
                Err(_err) => {
                    println!("[!] {:?}",file_path);
                    return;
                }
            };

        let filename = file_path.file_name().unwrap().to_str().unwrap();

        let index_entry = IndexEntry {
            filename: String::from(filename),
            filetype: FileType::get(get_file_extension(filename)),
            filepath: file_path.to_path_buf().canonicalize().unwrap(),
            keywords: keywords,
            last_modified_timestamp: meta
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };


        println!("Filepath (abs): {:?}",index_entry.filepath);


        match file_crawl_status{
            FileCrawlStatus::FirstFileIndex => {
                self.store_new_index(&index_entry);
            },
            FileCrawlStatus::FileChanged => {
                self.update_index(&index_entry);
            },
            FileCrawlStatus::FileNotChanged => unreachable!()
        }

    }

    fn check_needs_crawl(&self, file_path: &str, life_file_last_modified:u128)->FileCrawlStatus{
        let params = &[(":fp",&String::from(file_path))];

        let mut stmt = self._conn.prepare("SELECT last_modified_timestamp FROM search_index WHERE filepath=:fp").unwrap();

        match stmt.query_row(params,|r |{
            let ts: u64 = r.get(0)?;
            Ok(ts)
        }){
            Ok(timestamp) => {
                if (timestamp as u128) == life_file_last_modified{
                    FileCrawlStatus::FileNotChanged
                }else{
                    FileCrawlStatus::FileChanged
                }
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                FileCrawlStatus::FirstFileIndex
            },
            Err(_) => panic!("Sqlite Error wtf")
        }
    }

    pub fn start_crawl(&mut self, start_path: PathBuf) {
        // TODO: For later crawls we should really only check if modified TS has changed!!

        // We do BFS with queue
        let mut current_directory = start_path;
        loop {
            // get all in current if
            match fs::read_dir(&current_directory) {
                Ok(r) => {
                    for dir_res in r {
                        match dir_res {
                            Ok(dir) => {
                                let meta = dir.metadata().unwrap();
                                if meta.is_dir() {
                                    // put into queue
                                    if !EXCLUDE_DIRS.contains(&dir.file_name().to_str().unwrap()) {
                                        self._search_queue.add(dir.path()).unwrap();
                                    }
                                } else if meta.is_file() {
                                    let file_name = dir.file_name();
                                    let file_ext = get_file_extension(file_name.to_str().unwrap());
                                    if !file_ext.is_none()
                                        && INDEXABLE_FILE_EXTENSIONS
                                            .contains(&file_ext.unwrap().as_str())
                                    {
                                        println!("Indexing file: {:?}", dir.file_name());
                                        self.index_file(&dir.path());
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Inner error: {:?}", e)
                            }
                        }
                    }
                }
                Err(e) => println!("<Invalid path...> ({:?}", e.to_string()),
            }

            current_directory = match self._search_queue.remove() {
                Ok(cs) => cs,
                Err(_) => break,
            }
        }

        println!("Finished!")
    }
}
