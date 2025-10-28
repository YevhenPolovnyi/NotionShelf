/// Data models module
///
/// This module contains all data structures used throughout the application.
/// It's organized into separate submodules for better maintainability:
///
/// - `book`: Book-related data structures (BookInfo, NotionPage)
/// - `config`: Application configuration (Config)
pub mod book;
pub mod config;

// Re-export commonly used types for convenience
pub use book::{BookInfo, NotionPage};
pub use config::Config;
