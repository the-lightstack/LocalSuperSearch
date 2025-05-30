use clap::Parser;

mod storage;





fn main() {
    let cd = storage::CrawlDatabase::init();

    println!("Hello, world!");
}
