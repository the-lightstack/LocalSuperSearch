use std::{ops::Index, path::PathBuf};
use sqlite::Connection;
use std::fs;


extern crate queues;
use queues::*;

#[derive(Debug)]
enum FileType{
    Markdown = 1,
    Config,
    HTML,
    Python,
    JavaScript,
    Rust,
    CSOURCE,
    Presentation,
    PDF,
    LibreWriter
}

#[derive(Debug)]
struct IndexEntry{
    filename: String,

    filetype: FileType,
    filepath: PathBuf,
    keywords: String,
    last_modified_timestamp: usize,
}

pub struct CrawlDatabase{
    _conn: Connection,
    _search_queue: Queue<PathBuf>

}

// #[cfg(test)]
// mod tests{
//     use super::*;

//     #[test]
//     fn sth_works(){}

// }

const EXCLUDE_DIRS: &[&str] = &[".git",".yarn",".var","venv","__pycache__"];
impl CrawlDatabase{
    pub fn init()->Self{
        let conn = sqlite::open("test.db").unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS search_index (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            filename STRING NOT NULL,
            filetype INT NOT NULL,
            filepath STRING UNIQUE NOT NULL,
            keywords STRING,
            last_modified_timestamp TIMESTAMP
        ) ").unwrap();

        Self{
            _conn : conn,
            _search_queue: queue![]

        }
    }

    fn store_index(&self,ie: &IndexEntry ){
        self._conn.execute("INSERT INTO search_index ()").unwrap();

    }


    fn index_file(){}


    pub fn start_crawl(&mut self,start_path: PathBuf){
        // TODO: For later crawls we should really only check if modified TS has changed!!

        // We do BFS with queue
        let mut current_directory = start_path;
        loop {
            // get all in current if
            match fs::read_dir(&current_directory){
                Ok(r) =>
                {
                    for dir_res in r{
                        match dir_res{
                            Ok(dir) => {
                                let meta = dir.metadata().unwrap();
                                if meta.is_dir(){
                                    // put into queue
                                    self._search_queue.add(dir.path()).unwrap();
                                }else if meta.is_file(){
                                    println!("Indexing file: {:?}",dir.file_name());
                                }
                            },
                            Err(e) => {println!("Inner error: {:?}",e)}
                        }
                    }

                },
                Err(e) => println!("<Invalid path...> ({:?}",e.to_string())
            }

            current_directory = match self._search_queue.remove(){
                Ok(cs) => cs,
                Err(_) => break
            }
        }

        println!("Finished!")

    }
}
