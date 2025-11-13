/// UI rendering components
///
/// This module handles all terminal rendering logic using the ratatui library.
/// It renders different views based on the application state.
use color_eyre::Result;
use image;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;

use super::state::{AppState, UIState};

impl UIState {
    /// Main render function - delegates to appropriate view renderer
    ///
    /// This is the entry point for all rendering. It examines the current
    /// state and calls the appropriate rendering function.
    ///
    /// # Arguments
    ///
    /// * `frame` - Ratatui frame to render into
    /// * `epub_files` - List of EPUB file paths to display
    pub fn render(&mut self, frame: &mut Frame, epub_files: &[String]) {
        // Clone messages if present to avoid borrow issues
        let error_msg = if let AppState::Error(msg) = &self.state {
            Some(msg.clone())
        } else {
            None
        };

        let success_msg = if let AppState::Success(msg) = &self.state {
            Some(msg.clone())
        } else {
            None
        };

        let loading_msg = if let AppState::Loading(msg) = &self.state {
            Some(msg.clone())
        } else {
            None
        };

        match &self.state {
            AppState::DualView => self.render_dual_view(frame, epub_files),
            AppState::ConfirmAddToNotion => {
                // Render the base dual view first, then overlay the popup
                self.render_dual_view(frame, epub_files);
                self.render_confirmation_popup(frame);
            }
            AppState::Loading(_) => {
                // Render the base dual view first, then overlay loading popup
                self.render_dual_view(frame, epub_files);
                if let Some(msg) = loading_msg {
                    self.render_loading(frame, &msg);
                }
            }
            AppState::Success(_) => {
                // Render the base dual view first, then overlay success popup
                self.render_dual_view(frame, epub_files);
                if let Some(msg) = success_msg {
                    self.render_success(frame, &msg);
                }
            }
            AppState::Error(_) => {
                // Render the base dual view first, then overlay error popup
                self.render_dual_view(frame, epub_files);
                if let Some(msg) = error_msg {
                    self.render_error(frame, &msg);
                }
            }
        }

        // Overlay status message if present
        if let Some(ref message) = self.status_message {
            self.render_status_message(frame, message);
        }
    }

    /// Render the main dual-panel view
    ///
    /// This view shows:
    /// - Left panel: EPUB files menu (30%)
    /// - Right panel: Selected book details with actions and shortcuts (70%)
    fn render_dual_view(&mut self, frame: &mut Frame, epub_files: &[String]) {
        // Split screen into left menu and right details (full height)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left: EPUB files menu
                Constraint::Percentage(70), // Right: Book details
            ])
            .split(frame.area());

        // Render EPUB files menu on the left
        self.render_epub_files_panel(frame, main_chunks[0], epub_files);

        // Render book details on the right
        self.render_book_details_panel(frame, main_chunks[1]);
    }

    /// Render the EPUB files panel
    ///
    /// Shows a list of local EPUB files with the filename extracted.
    fn render_epub_files_panel(&mut self, frame: &mut Frame, area: Rect, epub_files: &[String]) {
        // Create list items with just filenames
        let items: Vec<ListItem> = epub_files
            .iter()
            .map(|path| {
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(path);
                ListItem::new(filename)
            })
            .collect();

        // Ensure selection is within bounds
        self.clamp_file_selection(epub_files.len());

        // Create title
        let title = format!(" EPUB Files ({}) ", epub_files.len());

        // Style
        let list_style = Style::default().fg(Color::White);
        let highlight_style = Style::default()
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD);

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(list_style)
            .highlight_style(highlight_style);

        frame.render_stateful_widget(list, area, &mut self.file_list_state);
    }

    /// Render the book details panel (right side)
    ///
    /// Shows book information in the right panel when a book is selected:
    /// - Book metadata (title, author, filename)
    /// - Cover image (or placeholder)
    /// - Comment input field
    fn render_book_details_panel(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref book) = self.selected_book {
            // Clone book data to avoid borrow checker issues
            let book_title = book.title.clone();
            let book_author = book.author.clone();
            let book_file_path = book.file_path.clone();
            let has_cover = book.cover_image.is_some();
            let cover_data = book.cover_image.clone();

            // Split details panel vertically
            let detail_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(20), // Book info + cover (doubled)
                    Constraint::Min(10),    // Comment input
                    Constraint::Length(15), // Action instructions with keyboard shortcuts
                ])
                .split(area);

            // Split book info section: Cover (left, fixed width) and Book Info (right, rest)
            let book_info_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(28),         // Cover: fixed width for image (doubled)
                    Constraint::Percentage(100), // Book Information: takes remaining space
                ])
                .split(detail_chunks[0]);

            // Render cover image (LEFT side)
            self.render_book_cover(frame, book_info_chunks[0], has_cover, cover_data);

            // Book details text (RIGHT side)
            let details_text = vec![
                Line::from(vec![
                    Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(book_title),
                ]),
                Line::from(vec![
                    Span::styled("Author: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(book_author),
                ]),
                Line::from(vec![
                    Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(
                        std::path::Path::new(&book_file_path)
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or(&book_file_path),
                    ),
                ]),
            ];

            let details = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .title(" Book Information ")
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(details, book_info_chunks[1]);

            // Comment input section
            // Always render comment field (even if empty) to support cursor positioning
            self.render_comment_with_cursor(frame, detail_chunks[1]);

            // Action instructions with keyboard shortcuts - varies by mode
            let actions_text = match self.input_mode {
                super::state::InputMode::Navigation => vec![
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Mode: NAVIGATION",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "[Enter] Add to Notion",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Navigation:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        "  j/k or ↑↓ - navigate files",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Actions:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        "  i - start editing comment",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(vec![Span::styled(
                        "  a - append to comment",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(vec![Span::styled(
                        "  q - quit",
                        Style::default().fg(Color::Red),
                    )]),
                ],
                super::state::InputMode::Editing => vec![
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Mode: EDITING",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "[Enter] Add to Notion",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Editing:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        "  Type - add text",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(vec![Span::styled(
                        "  ←→ - move cursor",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(vec![Span::styled(
                        "  Backspace/Del - delete",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(vec![Span::styled(
                        "  Home/End - jump",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "[Esc] Exit editing mode",
                        Style::default().fg(Color::Yellow),
                    )]),
                ],
            };

            let actions = Paragraph::new(actions_text)
                .alignment(Alignment::Left)
                .block(Block::default().title(" Actions ").borders(Borders::ALL));
            frame.render_widget(actions, detail_chunks[2]);
        } else {
            // No book selected - show placeholder
            let placeholder = Paragraph::new(vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "No book selected",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Select an EPUB file from the left panel",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Book Details ")
                    .borders(Borders::ALL),
            );
            frame.render_widget(placeholder, area);
        }
    }

    /// Render the book detail view
    ///
    /// Shows:
    /// Render comment field with cursor
    ///
    /// Shows the comment text with a terminal cursor at the current position.
    /// The cursor is managed by ratatui and will blink naturally.
    fn render_comment_with_cursor(&self, frame: &mut Frame, area: Rect) {
        // Determine what text to show
        let (display_text, text_style) = if self.user_comment.is_empty() {
            // Show placeholder when empty (only in navigation mode)
            match self.input_mode {
                super::state::InputMode::Navigation => (
                    "Press 'i' to start editing comment...",
                    Style::default().fg(Color::Gray),
                ),
                super::state::InputMode::Editing => ("", Style::default().fg(Color::Yellow)),
            }
        } else {
            // Show actual comment text with color based on mode
            (
                self.user_comment.as_str(),
                match self.input_mode {
                    super::state::InputMode::Navigation => Style::default().fg(Color::White),
                    super::state::InputMode::Editing => Style::default().fg(Color::Yellow),
                },
            )
        };

        // Create paragraph without wrap to keep cursor position simple
        let input = Paragraph::new(display_text).style(text_style).block(
            Block::default()
                .title(" Personal Comment/Review ")
                .borders(Borders::ALL),
        );

        frame.render_widget(input, area);

        // Set cursor position when in editing mode
        if matches!(self.input_mode, super::state::InputMode::Editing) {
            // Calculate the cursor position within the input area
            // +1 for left border, cursor position is relative to text start
            let cursor_x = area.x + self.comment_cursor as u16 + 1;
            let cursor_y = area.y + 1; // +1 for top border

            // Set cursor position using Ratatui's API
            frame.set_cursor_position(ratatui::layout::Position::new(cursor_x, cursor_y));
        }
    }
    /// Render book cover image or ASCII placeholder
    ///
    /// Attempts to render the actual cover image if available.
    /// Falls back to ASCII art if image rendering fails or no cover exists.
    fn render_book_cover(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        _has_cover: bool,
        cover_data: Option<Vec<u8>>,
    ) {
        if let Some(cover_bytes) = cover_data {
            // Try to render actual image
            let _ = self.try_render_image(frame, area, &cover_bytes);
        }
    }

    /// Attempt to render an actual image
    ///
    /// Uses the ratatui_image library to render the image in the terminal.
    /// This may fail if the terminal doesn't support the required protocols.
    fn try_render_image(&mut self, frame: &mut Frame, area: Rect, image_data: &[u8]) -> Result<()> {
        // Load image from bytes
        let img = image::load_from_memory(image_data)?;

        // Create rendering protocol
        let mut protocol = self.image_picker.new_resize_protocol(img);

        // Create inner area for image (leave room for border)
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        // Draw border
        let block = Block::default().title(" Cover ").borders(Borders::ALL);
        frame.render_widget(block, area);

        // Render image
        let image_widget = StatefulImage::new();
        frame.render_stateful_widget(image_widget, inner_area, &mut protocol);

        Ok(())
    }

    /// Render loading popup
    ///
    /// Shows a centered loading indicator overlay.
    fn render_loading(&self, frame: &mut Frame, message: &str) {
        let area = frame.area();

        // Create centered popup area
        let popup_width = 50.min(area.width.saturating_sub(4));
        let popup_height = 5;
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        // Render loading indicator with custom message
        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("⏳ {}", message),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Please Wait ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        frame.render_widget(loading, popup_area);
    }

    /// Render error popup
    ///
    /// Shows error message with instruction to dismiss.
    fn render_error(&self, frame: &mut Frame, error_msg: &str) {
        let area = frame.area();

        // Create centered popup area (larger for error messages)
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 8;
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        // Render error message
        let error_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "❌ Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                error_msg,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "[Enter/Esc] Close",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let error = Paragraph::new(error_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Error ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(error, popup_area);
    }

    /// Render success popup
    ///
    /// Shows success message after adding book to Notion.
    fn render_success(&self, frame: &mut Frame, success_msg: &str) {
        let area = frame.area();

        // Create centered popup area
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 8;
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        // Render success message
        let success_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "✅ Success!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                success_msg,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "[Enter/Esc] Close",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let success = Paragraph::new(success_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Success ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(success, popup_area);
    }

    /// Render confirmation popup for adding book to Notion
    ///
    /// Shows a centered popup asking user to confirm the action.
    fn render_confirmation_popup(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create centered popup area (larger than status message)
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 12;
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        // Get book info for display
        let book_title = self
            .selected_book
            .as_ref()
            .map(|b| b.title.as_str())
            .unwrap_or("Unknown");

        // Create popup content
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Add Book to Notion?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Book: ", Style::default().fg(Color::Gray)),
                Span::styled(book_title, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "This will create a new page in your Notion database.",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Y/Enter] ", Style::default().fg(Color::Green)),
                Span::styled("Confirm  ", Style::default().fg(Color::White)),
                Span::styled("[N/Esc] ", Style::default().fg(Color::Red)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]),
        ];

        let popup = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .title(" Confirmation ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        frame.render_widget(popup, popup_area);
    }

    /// Render status message popup
    ///
    /// Shows a temporary message overlay in the center of the screen.
    fn render_status_message(&self, frame: &mut Frame, message: &str) {
        let area = frame.area();

        // Create centered popup area
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 2 - 2,
            width: area.width / 2,
            height: 4,
        };

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        // Render status message
        let status = Paragraph::new(message)
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            .block(Block::default().title("Status").borders(Borders::ALL));
        frame.render_widget(status, popup_area);
    }
}
