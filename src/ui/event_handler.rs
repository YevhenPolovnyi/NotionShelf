/// Event handling for keyboard input
///
/// This module processes terminal events and translates them into
/// application actions based on the current UI state.
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use super::state::{AppState, InputMode, UIAction, UIState};

impl UIState {
    /// Handle a terminal event
    ///
    /// This is the main event processing entry point. It filters
    /// for key press events and delegates to state-specific handlers.
    ///
    /// # Arguments
    ///
    /// * `event` - Terminal event from crossterm
    ///
    /// # Returns
    ///
    /// Returns a `UIAction` indicating what the application should do
    pub fn handle_event(&mut self, event: Event) -> Result<UIAction> {
        if let Event::Key(key) = event {
            // Only process key press events, not releases
            if key.kind == KeyEventKind::Press {
                return self.handle_key_event(key);
            }
        }
        Ok(UIAction::None)
    }

    /// Handle a key press event
    ///
    /// Delegates to the appropriate handler based on current app state.
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<UIAction> {
        match &self.state {
            AppState::DualView => self.handle_dual_view_keys(key),
            AppState::ConfirmAddToNotion => self.handle_confirmation_popup_keys(key),
            AppState::Loading(_) => Ok(UIAction::None),
            AppState::Success(_) => {
                // Any key dismisses success popup
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                    self.state = AppState::DualView;
                }
                Ok(UIAction::None)
            }
            AppState::Error(_) => {
                // Any key dismisses error popup
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                    self.state = AppState::DualView;
                }
                Ok(UIAction::None)
            }
        }
    }

    /// Handle keys in confirmation popup
    ///
    /// Y/Enter: Confirm and add to Notion
    /// N/Esc: Cancel and return to DualView
    fn handle_confirmation_popup_keys(&mut self, key: KeyEvent) -> Result<UIAction> {
        match key.code {
            // Confirm - proceed with adding to Notion
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Ok(UIAction::ConfirmAddToNotion)
            }

            // Cancel - return to DualView
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state = AppState::DualView;
                Ok(UIAction::CancelAddToNotion)
            }

            _ => Ok(UIAction::None),
        }
    }

    /// Handle key events in the main dual-panel view
    ///
    /// Behavior depends on input mode:
    ///
    /// Navigation mode:
    /// - j/k/↑↓: Move selection, auto-load book details
    /// - i: Enter editing mode
    /// - Enter: Add book to Notion
    /// - q: Quit
    ///
    /// Editing mode:
    /// - Esc: Return to navigation mode
    /// - Typing: Add to comment field
    /// - Arrow keys: Move cursor
    fn handle_dual_view_keys(&mut self, key: KeyEvent) -> Result<UIAction> {
        match self.input_mode {
            InputMode::Navigation => self.handle_navigation_mode(key),
            InputMode::Editing => self.handle_editing_mode(key),
        }
    }

    /// Handle keys in navigation mode
    fn handle_navigation_mode(&mut self, key: KeyEvent) -> Result<UIAction> {
        match key.code {
            // Quit application
            KeyCode::Char('q') => {
                self.should_quit = true;
                Ok(UIAction::Quit)
            }

            // Enter editing mode
            KeyCode::Char('i') | KeyCode::Char('a') => {
                self.input_mode = InputMode::Editing;
                // 'a' moves cursor to end for "append" mode
                if key.code == KeyCode::Char('a') {
                    self.comment_cursor = self.user_comment.chars().count();
                }
                Ok(UIAction::None)
            }

            // Move selection down - triggers auto-load of book details
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_file_selection_down();
                Ok(UIAction::SelectFile)
            }

            // Move selection up - triggers auto-load of book details
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_file_selection_up();
                Ok(UIAction::SelectFile)
            }

            // Show confirmation popup before adding to Notion
            KeyCode::Enter => Ok(UIAction::ShowAddToNotionConfirmation),

            _ => Ok(UIAction::None),
        }
    }

    /// Handle keys in editing mode
    fn handle_editing_mode(&mut self, key: KeyEvent) -> Result<UIAction> {
        match key.code {
            // Exit editing mode
            KeyCode::Esc => {
                self.input_mode = InputMode::Navigation;
                Ok(UIAction::None)
            }

            // Character input - add to comment
            KeyCode::Char(c) => {
                let byte_index = self.char_to_byte_index(self.comment_cursor);
                self.user_comment.insert(byte_index, c);
                self.comment_cursor += 1;
                Ok(UIAction::None)
            }

            // Backspace - remove character before cursor
            KeyCode::Backspace => {
                if self.comment_cursor > 0 {
                    self.comment_cursor -= 1;
                    let byte_index = self.char_to_byte_index(self.comment_cursor);
                    self.user_comment.remove(byte_index);
                }
                Ok(UIAction::None)
            }

            // Delete - remove character after cursor
            KeyCode::Delete => {
                let byte_index = self.char_to_byte_index(self.comment_cursor);
                if byte_index < self.user_comment.len() {
                    self.user_comment.remove(byte_index);
                }
                Ok(UIAction::None)
            }

            // Left arrow - move cursor left
            KeyCode::Left => {
                if self.comment_cursor > 0 {
                    self.comment_cursor -= 1;
                }
                Ok(UIAction::None)
            }

            // Right arrow - move cursor right
            KeyCode::Right => {
                let char_count = self.user_comment.chars().count();
                if self.comment_cursor < char_count {
                    self.comment_cursor += 1;
                }
                Ok(UIAction::None)
            }

            // Home - jump to start
            KeyCode::Home => {
                self.comment_cursor = 0;
                Ok(UIAction::None)
            }

            // End - jump to end
            KeyCode::End => {
                self.comment_cursor = self.user_comment.chars().count();
                Ok(UIAction::None)
            }

            // Enter in editing mode - show confirmation popup
            KeyCode::Enter => Ok(UIAction::ShowAddToNotionConfirmation),

            _ => Ok(UIAction::None),
        }
    }
}
