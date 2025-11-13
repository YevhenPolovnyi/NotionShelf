/// UI state management
///
/// This module defines all the state structures and transitions
/// for the terminal user interface.
use ratatui::widgets::ListState;
use ratatui_image::picker::Picker;

use crate::models::BookInfo;

/// Application view states
///
/// These represent the different screens or modes the application can be in.
/// The UI renders differently based on the current state.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Main dual-panel view: EPUB files menu (left) and book details (right)
    DualView,

    /// Confirmation popup before adding book to Notion
    ConfirmAddToNotion,

    /// Loading indicator during async operations with status message
    Loading(String),

    /// Success message after adding to Notion
    Success(String),

    /// Error display with message
    Error(String),
}

/// Input mode for dual-panel view
///
/// Determines how keyboard input is interpreted:
/// - Navigation: j/k for file selection, other keys for actions
/// - Editing: All printable keys go into comment field
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    /// Navigation mode: browse files with j/k
    Navigation,

    /// Editing mode: type into comment field
    Editing,
}

/// User actions triggered by UI events
///
/// These represent high-level actions that the application
/// should perform in response to user input.
#[derive(Debug, Clone)]
pub enum UIAction {
    /// No action needed
    None,

    /// User wants to quit the application
    Quit,

    /// User selected an EPUB file to view details
    SelectFile,

    /// User wants to add the current book to Notion (show confirmation popup)
    ShowAddToNotionConfirmation,

    /// User confirmed adding to Notion
    ConfirmAddToNotion,

    /// User cancelled adding to Notion
    CancelAddToNotion,
}

/// Main UI state container
///
/// This structure holds all the state needed for rendering
/// and managing the terminal user interface.
pub struct UIState {
    /// Current view/screen state
    pub state: AppState,

    /// Input mode: Navigation or Editing
    pub input_mode: InputMode,

    /// Selection state for the EPUB files list
    pub file_list_state: ListState,

    /// Currently selected book for detail view
    pub selected_book: Option<BookInfo>,

    /// User's comment/review text for the book
    pub user_comment: String,

    /// Cursor position in the comment field (in characters, not bytes)
    pub comment_cursor: usize,

    /// Temporary status message to display
    pub status_message: Option<String>,

    /// Flag indicating the application should quit
    pub should_quit: bool,

    /// Image rendering protocol picker
    pub image_picker: Picker,
}

impl UIState {
    /// Create a new UI state with default values
    ///
    /// Initializes all state with sensible defaults:
    /// - Loading state initially
    /// - First item in each list selected
    /// - No book selected
    /// - All text fields empty
    pub fn new() -> Self {
        let mut file_list_state = ListState::default();
        file_list_state.select(Some(0));

        Self {
            state: AppState::Loading("Loading...".to_string()),
            input_mode: InputMode::Navigation, // Start in navigation mode
            file_list_state,
            selected_book: None,
            user_comment: String::new(),
            comment_cursor: 0,
            status_message: None,
            should_quit: false,
            image_picker: Picker::from_query_stdio()
                .unwrap_or_else(|_| Picker::from_fontsize((8, 16))),
        }
    }

    /// Set the error state with a message

    /// Return to the main file list view
    ///
    /// This clears the selected book and any entered comment.
    /// Note: Currently unused as we stay in DualView, but kept for potential future use.
    #[allow(dead_code)]
    pub fn return_to_file_list(&mut self) {
        self.state = AppState::DualView;
        self.selected_book = None;
        self.user_comment.clear();
        self.comment_cursor = 0;
    }

    /// Set an error state with a message
    pub fn set_error(&mut self, error: String) {
        self.state = AppState::Error(error);
    }

    /// Set a temporary status message
    #[allow(dead_code)]
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear the status message
    #[allow(dead_code)]
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Get the currently selected file index (if any)
    pub fn get_selected_file_index(&self) -> Option<usize> {
        self.file_list_state.selected()
    }

    /// Move file selection down in the list
    pub fn move_file_selection_down(&mut self) {
        let selected = self.file_list_state.selected().unwrap_or(0);
        self.file_list_state.select(Some(selected + 1));
    }

    /// Move file selection up in the list
    pub fn move_file_selection_up(&mut self) {
        let selected = self.file_list_state.selected().unwrap_or(0);
        if selected > 0 {
            self.file_list_state.select(Some(selected - 1));
        }
    }

    /// Ensure file list selection is within bounds
    ///
    /// This should be called after the file list changes
    /// to prevent out-of-bounds selections.
    pub fn clamp_file_selection(&mut self, file_count: usize) {
        if let Some(selected) = self.file_list_state.selected() {
            if selected >= file_count && file_count > 0 {
                self.file_list_state.select(Some(file_count - 1));
            } else if file_count == 0 {
                self.file_list_state.select(None);
            }
        }
    }

    /// Convert character position to byte position in the comment string
    ///
    /// This is necessary because Rust strings are UTF-8 encoded,
    /// and character indices don't directly map to byte indices.
    pub fn char_to_byte_index(&self, char_index: usize) -> usize {
        self.user_comment
            .char_indices()
            .nth(char_index)
            .map(|(byte_index, _)| byte_index)
            .unwrap_or(self.user_comment.len())
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}
