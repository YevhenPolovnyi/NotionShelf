use color_eyre::Result;

// Module declarations
mod app;
mod models;
mod services;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Install color_eyre panic and error handlers
    color_eyre::install()?;

    // Initialize terminal with ratatui::init()
    let terminal = ratatui::init();

    // Create and run the app
    let mut app = App::new(terminal).await?;
    let app_result = app.run().await;

    // Always restore terminal with ratatui::restore(), even if there was an error
    ratatui::restore();

    // Return the result from run()
    app_result
}
