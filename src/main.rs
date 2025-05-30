
use std::{env::home_dir, path::PathBuf, process::exit};

use clap::Parser;

mod file_index;
mod crawl;

#[derive(Parser)]
#[command(name= "Indexed Search")]
#[command(version="0.0.1")]
struct Cli{
    #[arg(short,long,value_name="crawl")]
    crawl_dir: Option<String>,

    search_term: Option<String>,
}

fn database_location()->String{
   String::from(home_dir().unwrap().join(".local/share/local_super_search_index.db").to_str().unwrap())
}


fn main() {
    let args = Cli::parse();

    // Check if crawl parameter was given
    match args.crawl_dir{
        Some(crawl_path) =>{
            let mut crawl_database = crawl::CrawlDatabase::init(&database_location());

            println!("Starting crawl for path [{:?}]",crawl_path);
            crawl_database.start_crawl(PathBuf::from(crawl_path));

            println!("Finished Indexing!");
            println!("You can now quickly search with: is <TERM>");
            exit(0);
        },
        None => {}
    }

    // Otherwise activate search mode


}
