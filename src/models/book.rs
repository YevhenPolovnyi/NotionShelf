/// Book-related data models
///
/// This module contains all data structures related to book information,
/// whether parsed from EPUB files or fetched from Notion.

/// Represents comprehensive information about a book parsed from an EPUB file
///
/// This structure holds all relevant book metadata including:
/// - Basic information (title, author)
/// - Cover image data (if available)
/// - File system path to the original EPUB file
#[derive(Debug, Clone)]
pub struct BookInfo {
    /// The title of the book extracted from EPUB metadata
    pub title: String,

    /// The author name extracted from EPUB metadata
    pub author: String,

    /// Optional cover image data as raw bytes
    /// Supports common image formats (JPEG, PNG, GIF, WebP, BMP)
    pub cover_image: Option<Vec<u8>>,

    /// Full file system path to the EPUB file
    pub file_path: String,
}

/// Represents a newly created Notion page response
///
/// This structure captures the essential data returned by Notion's API
/// when a new page (book entry) is successfully created.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NotionPage {
    /// Unique identifier for the created Notion page
    pub id: String,

    /// Direct URL to access the created page
    pub url: String,
}
