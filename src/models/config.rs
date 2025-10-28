/// Application configuration management
///
/// This module handles loading and validation of configuration from environment variables.
/// Configuration is typically loaded from a `.env` file in the project root.
use color_eyre::{eyre::Context, Result};
use std::path::PathBuf;

/// Application configuration loaded from environment variables
///
/// This structure contains all the settings needed to run the application:
/// - Notion API credentials and database information
/// - Local file system paths
/// - Property name mappings for Notion database columns
/// - Behavioral flags
#[derive(Debug, Clone)]
pub struct Config {
    /// Notion API integration token (secret key)
    /// Format: "secret_xxxxxxxxxxxxxxxxxxxxxxxxxxxx"
    /// Obtained from: https://www.notion.so/my-integrations
    pub notion_api_key: String,

    /// Unique identifier for the Notion database to use
    /// This is the 32-character ID from the database URL
    /// The integration must have access to this database
    pub notion_database_id: String,

    /// Local directory path where EPUB files are stored
    /// The application will scan this directory for .epub files
    pub epub_directory_path: PathBuf,

    /// Name of the Notion database property for book titles
    /// Default: "Title"
    pub notion_title_property: String,

    /// Name of the Notion database property for author names
    /// Default: "Author"
    pub notion_author_property: String,

    /// Name of the Notion database property for cover images
    /// Default: "Cover"
    pub notion_cover_property: String,

    /// Whether to delete EPUB files after successfully adding them to Notion
    /// Default: true (for backward compatibility)
    pub delete_files_after_adding: bool,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// This method:
    /// 1. Loads the `.env` file using dotenv
    /// 2. Reads and validates all required environment variables
    /// 3. Provides helpful error messages if configuration is missing or invalid
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `.env` file is missing
    /// - Required environment variables are not set
    /// - Environment variables contain placeholder values
    /// - The EPUB directory path doesn't exist
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use notionshelf::models::config::Config;
    ///
    /// let config = Config::load().expect("Failed to load configuration");
    /// println!("Using database: {}", config.notion_database_id);
    /// ```
    pub fn load() -> Result<Self> {
        // Load environment variables from .env file
        // Try to find .env in the current directory first, then in the executable's directory
        let env_result = dotenv::dotenv();

        if env_result.is_err() {
            // If .env not found in current directory, try the executable's directory
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let env_file = exe_dir.join(".env");
                    if env_file.exists() {
                        dotenv::from_path(&env_file).ok();
                    }
                }
            }
        }

        // Verify that the .env file was loaded successfully
        if std::env::var("NOTION_API_KEY").is_err() {
            return Err(color_eyre::eyre::eyre!(
                "❌ Failed to load .env file. Make sure the file exists in the project root directory."
            ));
        }

        // Load and validate Notion API key
        let notion_api_key = std::env::var("NOTION_API_KEY")
            .context("❌ NOTION_API_KEY not found in .env file. Add your Notion API token.")?;

        if notion_api_key == "your_notion_api_key_here" {
            return Err(color_eyre::eyre::eyre!(
                "❌ NOTION_API_KEY not configured! Replace 'your_notion_api_key_here' with your actual API key.\n\
                💡 Get your key from: https://www.notion.so/my-integrations"
            ));
        }

        // Load and validate Notion database ID
        let notion_database_id = std::env::var("NOTION_DATABASE_ID")
            .context("❌ NOTION_DATABASE_ID not found in .env file. Add your database ID.")?;

        if notion_database_id == "your_database_id_here" {
            return Err(color_eyre::eyre::eyre!(
                "❌ NOTION_DATABASE_ID not configured! Replace 'your_database_id_here' with your database ID.\n\
                💡 Run ./find_notion_databases.sh to find available databases."
            ));
        }

        // Load and validate EPUB directory path
        let epub_directory_path = std::env::var("EPUB_DIRECTORY_PATH")
            .context("EPUB_DIRECTORY_PATH environment variable not found")?;

        let epub_directory_path = PathBuf::from(epub_directory_path);

        if !epub_directory_path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "EPUB directory path does not exist: {}",
                epub_directory_path.display()
            ));
        }

        // Load Notion property names with sensible defaults
        let notion_title_property =
            std::env::var("NOTION_TITLE_PROPERTY").unwrap_or_else(|_| "Title".to_string());

        let notion_author_property =
            std::env::var("NOTION_AUTHOR_PROPERTY").unwrap_or_else(|_| "Author".to_string());

        let notion_cover_property =
            std::env::var("NOTION_COVER_PROPERTY").unwrap_or_else(|_| "Cover".to_string());

        // Load file deletion preference (default to true for backward compatibility)
        let delete_files_after_adding = std::env::var("DELETE_FILES_AFTER_ADDING")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            .parse::<bool>()
            .unwrap_or(true);

        Ok(Config {
            notion_api_key,
            notion_database_id,
            epub_directory_path,
            notion_title_property,
            notion_author_property,
            notion_cover_property,
            delete_files_after_adding,
        })
    }
}
