
enum FileType{
    Markdown,
    Config,
    HTML,
    Python,
    JavaScript,
    Rust,
    C_Source,
    Presentation,
    PDF,
    LibreWriter
}

struct IndexEntry{
    filename: str,

    filetype: FileType,
    filepath: PathBuf,
    keywords: str,
    last_modified_timestamp: usize,
}