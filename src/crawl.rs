use std::{ path::PathBuf};
use sqlite::Connection;
use std::fs;


extern crate queues;
use queues::*;

use crate::file_index::extract_keywords;

#[derive(Debug, Clone, Copy)]
enum FileType{
    Markdown = 1,
    Config,
    Web,
    Python,
    JavaScript,
    Rust,
    CSOURCE,
    Presentation,
    PDF,
    LibreWriter,
    Excel,
    Plain,
    Container,


    Unknown,
}
impl FileType {
    pub fn get(file_ext:&str) -> Self{
        match file_ext{
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

            _ => Self::Unknown


        }

    }
}




impl From<usize> for FileType {
    fn from(value: usize) -> Self {
        value.into()
    }
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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn db_init_and_insert(){
        let cdb = CrawlDatabase::init(":memory:");
        let ie = IndexEntry{
            filename: String::from("test_file_name.txt"),
            filetype: FileType::Config,
            filepath: PathBuf::from("/test/test_file_name.txt"),
            keywords: String::from("test,config,vector"),
            last_modified_timestamp: 92738728374,
        };

        cdb.store_index(&ie);
    }

    #[test]
    fn get_file_extension_test(){
        let cdb = CrawlDatabase::init(":memory:");
        assert_eq!(None,cdb.get_file_extension("nothing"));
        assert_eq!(None,cdb.get_file_extension("nothing."));
        assert_eq!(Some("js"),cdb.get_file_extension("file.js"));
        assert_eq!(Some("config"),cdb.get_file_extension("big_filename.config"));



    }

}

const INDEXABLE_FILE_EXTENSIONS: &[&str] = &["md","yml","yaml","json","config","toml","xml","html","htmx","css","py","js","ts","rs","c","cpp","h","hpp","ppt","pptx","pps","ppsx","pot","potx","odp","odkey","doc","docx","dot","dotx","odt","ott","pages","rtf","txt","pdf"];
// const INDEXABLE_FILE_EXTENSIONS: &[&str] = &["pdf","ppt","pptx","py","js","ts","rs","c","cpp","h","hpp","md","txt","html","css","config","json","toml","yaml","yml","csv"];
const EXCLUDE_DIRS: &[&str] = &[".git",".yarn",".var","venv","__pycache__"];
impl CrawlDatabase{
    pub fn init(path:&str)->Self{
        let conn = sqlite::open(path).unwrap();
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
        let query = "INSERT INTO search_index
                    (filename, filetype, filepath,keywords,last_modified_timestamp)
                    VALUES (?,?,?,?,?)";
        let mut statement = self._conn.prepare(query).unwrap();

        // make more elegant/dynamic later ...

        statement.bind((1,ie.filename.as_str())).unwrap();
        statement.bind((2,ie.filetype as i64)).unwrap();
        statement.bind((3, ie.filepath.to_str())).unwrap();
        statement.bind((4, ie.keywords.as_str())).unwrap();
        statement.bind((5,ie.last_modified_timestamp as i64)).unwrap();

        statement.next().unwrap();
    }

    fn get_file_extension<'a>(&self,name:&'a str)->Option<&'a str>{
        if !name.contains("."){
            None
        }else{
            let ext = name.split(".").last()?;
            if ext.is_empty(){
                None
            }else{
                Some(name.split(".").last()?)
            }
        }
    }

    fn index_file(&self,file_path: &PathBuf){
        let meta = fs::metadata(file_path).expect("Expected to be able to read MetaData on file");
        extract_keywords(file_path);

    }


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
                                    if !EXCLUDE_DIRS.contains(&dir.file_name().to_str().unwrap()){
                                        self._search_queue.add(dir.path()).unwrap();
                                    }
                                }else if meta.is_file(){
                                    let file_name = dir.file_name();
                                    let file_ext = self.get_file_extension(file_name.to_str().unwrap());
                                    if !file_ext.is_none() && INDEXABLE_FILE_EXTENSIONS.contains(&file_ext.unwrap()){
                                        println!("Indexing file: {:?}",dir.file_name());
                                        self.index_file(&dir.path());
                                    }
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
