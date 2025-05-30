
use std::path::PathBuf;

use clap::Parser;

mod crawl;





fn main() {
    let mut cd = crawl::CrawlDatabase::init();
    println!("Starting crawl [...]");
    cd.start_crawl(PathBuf::from("."));
}
