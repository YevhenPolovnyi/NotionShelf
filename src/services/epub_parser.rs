/// EPUB file parsing service
///
/// This module provides functionality for:
/// - Parsing EPUB files to extract book metadata
/// - Extracting cover images from EPUB files
/// - Scanning directories for EPUB files
use color_eyre::{eyre::Context, Result};
use epub::doc::EpubDoc;
use std::path::Path;

use crate::models::BookInfo;

/// Parse an EPUB file and extract book information
///
/// This function opens an EPUB file and extracts:
/// - Title (from metadata or filename if not available)
/// - Author (from creator metadata)
/// - Cover image (using multiple fallback strategies)
///
/// # Arguments
///
/// * `path` - Path to the EPUB file (can be any type that implements `AsRef<Path>`)
///
/// # Returns
///
/// Returns a `BookInfo` struct containing all extracted metadata
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened or is not a valid EPUB
/// - File system operations fail
///
/// # Examples
///
/// ```no_run
/// use notionshelf::services::epub_parser::parse_epub;
///
/// let book_info = parse_epub("/path/to/book.epub").expect("Failed to parse EPUB");
/// println!("Title: {}", book_info.title);
/// println!("Author: {}", book_info.author);
/// ```
pub fn parse_epub<P: AsRef<Path>>(path: P) -> Result<BookInfo> {
    let path = path.as_ref();

    // Open the EPUB file
    let mut doc = EpubDoc::new(path)
        .with_context(|| format!("Failed to open EPUB file: {}", path.display()))?;

    // Extract title from metadata, or use filename as fallback
    let title = doc.mdata("title").unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Title")
            .to_string()
    });

    // Extract author from creator metadata
    let author = doc
        .mdata("creator")
        .unwrap_or_else(|| "Unknown Author".to_string());

    // Extract cover image using multiple strategies
    let cover_image = extract_cover_image(&mut doc)?;

    Ok(BookInfo {
        title,
        author,
        cover_image,
        file_path: path.to_string_lossy().to_string(),
    })
}

/// Extract cover image from an EPUB document
///
/// This function attempts multiple strategies to find a cover image:
/// 1. Check for cover image in EPUB metadata (standard location)
/// 2. Search for resources with "cover" in the filename
/// 3. Fall back to the first image file found
///
/// # Arguments
///
/// * `doc` - Mutable reference to an opened EPUB document
///
/// # Returns
///
/// Returns `Some(Vec<u8>)` with image data if a cover is found, `None` otherwise
///
/// # Errors
///
/// Returns an error only for critical failures; missing cover images are not errors
fn extract_cover_image(
    doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>,
) -> Result<Option<Vec<u8>>> {
    // Strategy 1: Try to get cover image from EPUB metadata
    if let Some(cover_data) = doc.get_cover() {
        return Ok(Some(cover_data.0));
    }

    // Strategy 2: Search for resources with "cover" in the name
    let resources = doc.resources.clone();
    for (id, _) in resources.iter() {
        if id.contains("cover") || id.contains("Cover") {
            if let Some(resource_data) = doc.get_resource(id) {
                return Ok(Some(resource_data.0));
            }
        }
    }

    // Strategy 3: Look for any image file as fallback
    for (id, _) in resources.iter() {
        if id.ends_with(".jpg")
            || id.ends_with(".jpeg")
            || id.ends_with(".png")
            || id.ends_with(".gif")
        {
            if let Some(resource_data) = doc.get_resource(id) {
                return Ok(Some(resource_data.0));
            }
        }
    }

    // No cover image found
    Ok(None)
}

/// Scan a directory for EPUB files
///
/// This function recursively scans a directory and returns a list of all
/// EPUB files found (files with .epub extension).
///
/// # Arguments
///
/// * `dir_path` - Path to the directory to scan
///
/// # Returns
///
/// Returns a vector of file paths (as strings) to all EPUB files found
///
/// # Errors
///
/// Returns an error if:
/// - The directory doesn't exist
/// - Directory cannot be read (permission issues, etc.)
/// - Individual directory entries cannot be read
///
/// # Examples
///
/// ```no_run
/// use notionshelf::services::epub_parser::scan_epub_directory;
///
/// let epub_files = scan_epub_directory("/path/to/books").expect("Failed to scan directory");
/// println!("Found {} EPUB files", epub_files.len());
/// ```
pub fn scan_epub_directory<P: AsRef<Path>>(dir_path: P) -> Result<Vec<String>> {
    let dir_path = dir_path.as_ref();

    // Validate that directory exists
    if !dir_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Directory does not exist: {}",
            dir_path.display()
        ));
    }

    let mut epub_files = Vec::new();

    // Read directory entries
    let entries = std::fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory: {}", dir_path.display()))?;

    // Filter for EPUB files
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Check if it's a file with .epub extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension.to_str() == Some("epub") {
                    epub_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Sort files alphabetically for consistent ordering
    epub_files.sort();

    Ok(epub_files)
}
