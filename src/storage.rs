use std::path::PathBuf;
use sqlite::Connection;

#[derive(Debug)]
pub struct CrawlEntry{
    path: PathBuf,
    file_extension: String,
}

pub struct CrawlDatabase{
    _conn: Connection
}

impl CrawlDatabase{
    pub fn init()->Self{
        Self{
            _conn : sqlite::open("::memory::").unwrap()
        }
    }
}
