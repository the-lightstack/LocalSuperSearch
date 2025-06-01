use crate::crawl::CrawlDatabase;

// TODO: we should keep weights of keywords so that multiple keywords have better SINGLE match

pub fn search_through_database(cdb: &CrawlDatabase, search_term: String) {
    // Cleaning up search term
    // search_term.trim().split_whitespace()

    cdb.search_keyword(&search_term);
}
