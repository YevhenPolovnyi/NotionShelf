/// Services module
/// 
/// This module contains all business logic and external service integrations:
/// 
/// - `epub_parser`: EPUB file parsing and directory scanning
/// - `notion_client`: Notion API client and operations

pub mod epub_parser;
pub mod notion_client;

// Re-export commonly used functions and types
pub use epub_parser::{parse_epub, scan_epub_directory};
pub use notion_client::NotionClient;
