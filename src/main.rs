
use std::path::PathBuf;

use clap::Parser;

mod file_index;
mod crawl;





fn main() {
    let mut cd = crawl::CrawlDatabase::init("./out.db");
    println!("Starting crawl [...]");
    cd.start_crawl(PathBuf::from("./testhome"));
}
