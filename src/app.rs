use color_eyre::{eyre::Context, Result};
use crossterm::event::{self};
use ratatui::DefaultTerminal;

use crate::{
    models::Config,
    services::{parse_epub, scan_epub_directory, NotionClient},
    ui::{AppState, UIAction, UIState},
};

pub struct App {
    /// Application configuration loaded from environment
    config: Config,

    /// Notion API client for database operations
    notion_client: NotionClient,

    /// UI state including current view and user input
    ui: UIState,

    /// List of EPUB file paths found in the configured directory
    epub_files: Vec<String>,

    /// Ratatui terminal instance for rendering
    terminal: DefaultTerminal,
}

impl App {
    pub async fn new(terminal: DefaultTerminal) -> Result<Self> {
        // Step 1: Load configuration from .env file
        let config = Config::load().context("Failed to load configuration")?;

        // Step 2: Create Notion client and verify connection
        let notion_client = NotionClient::new(&config);
        notion_client
            .test_connection()
            .await
            .context("Failed to connect to Notion API")?;

        // Step 3: Scan EPUB directory for files
        let epub_files = scan_epub_directory(&config.epub_directory_path)
            .context("Failed to scan EPUB directory")?;

        // Initialize UI state
        let mut ui = UIState::new();
        ui.state = AppState::DualView;

        // Step 4: Load first book details if available
        let selected_book = if !epub_files.is_empty() {
            match parse_epub(&epub_files[0]) {
                Ok(book_info) => Some(book_info),
                Err(_) => None, // Silently ignore parse errors for first book
            }
        } else {
            None
        };
        ui.selected_book = selected_book;

        Ok(App {
            config,
            notion_client,
            ui,
            epub_files,
            terminal,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Render current UI state
            self.terminal
                .draw(|frame| self.ui.render(frame, &self.epub_files))
                .context("Failed to draw terminal")?;

            // Read the next event (blocks until an event occurs)
            // This is more efficient than polling - zero CPU usage while waiting
            let event = event::read().context("Failed to read event")?;
            let action = self.ui.handle_event(event)?;

            // Process the action
            match action {
                UIAction::Quit => break,

                UIAction::SelectFile => {
                    if let Some(index) = self.ui.get_selected_file_index() {
                        if index < self.epub_files.len() {
                            self.handle_file_selection(index).await?;
                        }
                    }
                }

                UIAction::ShowAddToNotionConfirmation => {
                    // Show confirmation popup
                    self.ui.state = AppState::ConfirmAddToNotion;
                }

                UIAction::ConfirmAddToNotion => {
                    // User confirmed - proceed with adding to Notion
                    self.handle_add_to_notion().await?;
                }

                UIAction::CancelAddToNotion => {
                    // User cancelled - already handled in event handler
                    // State is already set back to DualView
                }

                UIAction::None => {}
            }

            // Check if user wants to quit
            if self.ui.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle EPUB file selection
    ///
    /// When a user navigates to an EPUB file:
    /// 1. Parse the EPUB file to extract metadata
    /// 2. Update selected book (stays in DualView, shows details in right panel)
    /// 3. Clear comment field for new book
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the selected file in the epub_files list
    async fn handle_file_selection(&mut self, index: usize) -> Result<()> {
        let file_path = &self.epub_files[index];

        // Parse the EPUB file
        match parse_epub(file_path) {
            Ok(book_info) => {
                // Success: update selected book in right panel
                self.ui.selected_book = Some(book_info);
                // Stay in DualView state
                self.ui.state = AppState::DualView;
                // Clear comment for new book
                self.ui.user_comment.clear();
                self.ui.comment_cursor = 0;
            }
            Err(e) => {
                // Error: show error message
                let error_msg = format!("Failed to parse EPUB file: {}", e);
                self.ui.set_error(error_msg);
            }
        }

        Ok(())
    }

    /// Handle adding the current book to Notion
    ///
    /// This is the main workflow for adding a book:
    /// 1. Show loading indicator
    /// 2. Upload book metadata and cover to Notion
    /// 3. Optionally delete the EPUB file if configured
    /// 4. Refresh the file list
    /// 5. Show success or error message
    /// 6. Return to file list view
    async fn handle_add_to_notion(&mut self) -> Result<()> {
        if let Some(ref book) = self.ui.selected_book.clone() {
            // Show initial loading message
            self.ui.state = AppState::Loading("Preparing to add book...".to_string());
            self.terminal
                .draw(|frame| self.ui.render(frame, &self.epub_files))
                .context("Failed to draw loading screen")?;

            let user_comment = self.ui.user_comment.clone();

            // Update status: uploading cover
            if book.cover_image.is_some() {
                self.ui.state = AppState::Loading("Uploading cover image...".to_string());
                self.terminal
                    .draw(|frame| self.ui.render(frame, &self.epub_files))
                    .context("Failed to draw loading screen")?;
            }

            // Update status: creating page
            self.ui.state = AppState::Loading("Creating page in Notion...".to_string());
            self.terminal
                .draw(|frame| self.ui.render(frame, &self.epub_files))
                .context("Failed to draw loading screen")?;

            match self
                .notion_client
                .create_book_page(book, &user_comment)
                .await
            {
                Ok(_notion_page) => {
                    // Delete the original file only if configured to do so
                    let success_msg = if self.config.delete_files_after_adding {
                        if let Err(e) = std::fs::remove_file(&book.file_path) {
                            let error_msg = format!("Failed to delete file: {}", e);
                            self.ui.set_error(error_msg);
                            return Ok(());
                        }

                        // Update the file list only if we deleted files
                        self.refresh_file_list()?;

                        format!(
                            "✅ Successfully added '{}' to Notion and deleted file!",
                            book.title
                        )
                    } else {
                        format!(
                            "✅ Successfully added '{}' to Notion! (File kept)",
                            book.title
                        )
                    };

                    // Clear comment
                    self.ui.user_comment.clear();
                    self.ui.comment_cursor = 0;

                    // Show success popup (user needs to press Enter/Esc to dismiss)
                    self.ui.state = AppState::Success(success_msg);
                }
                Err(e) => {
                    let error_msg = format!("❌ Failed to add book to Notion: {}", e);
                    self.ui.set_error(error_msg);
                }
            }
        }

        Ok(())
    }

    /// Refresh the EPUB file list
    ///
    /// Re-scans the EPUB directory and updates the file list.
    /// Also adjusts the UI selection to stay within bounds.
    ///
    /// This is called after deleting a file to update the display.
    fn refresh_file_list(&mut self) -> Result<()> {
        // Rescan the directory
        self.epub_files = scan_epub_directory(&self.config.epub_directory_path)
            .context("Failed to refresh EPUB file list")?;

        // Ensure selection is within bounds
        self.ui.clamp_file_selection(self.epub_files.len());

        Ok(())
    }
}
