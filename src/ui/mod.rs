pub mod event_handler;
pub mod renderer;
/// User Interface module
///
/// This module contains all UI-related code organized into logical components:
///
/// - `state`: UI state management and data structures
/// - `event_handler`: Keyboard input processing and event handling
/// - `renderer`: Terminal rendering using ratatui
///
/// Together, these components provide a complete terminal UI experience
/// with dual-panel view, book details, search, and status messages.
pub mod state;

// Re-export the main UI state and action types
pub use state::{AppState, UIAction, UIState};
