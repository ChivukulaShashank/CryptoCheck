pub mod app;
pub mod ui;

use std::io::{self, stdout};
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
pub use app::App;

/// Runs the interactive Terminal User Interface event loop
pub fn run_tui(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Terminal setup
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Set panic hook to ensure terminal raw mode is restored if a crash occurs
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_panic(panic_info);
    }));

    // 3. Main event & rendering loop
    let tick_rate = Duration::from_millis(60); // ~16 FPS smooth refresh

    loop {
        // Draw frame
        terminal.draw(|f| ui::render(f, app))?;

        // Handle inputs with non-blocking poll
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Tab | KeyCode::Right => {
                            app.current_tab = (app.current_tab + 1) % 4;
                        }
                        KeyCode::BackTab | KeyCode::Left => {
                            app.current_tab = if app.current_tab == 0 { 3 } else { app.current_tab - 1 };
                        }
                        KeyCode::Char('1') => app.current_tab = 0,
                        KeyCode::Char('2') => app.current_tab = 1,
                        KeyCode::Char('3') => app.current_tab = 2,
                        KeyCode::Char('4') => app.current_tab = 3,
                        KeyCode::Char(' ') => app.toggle_running(),
                        KeyCode::Char('m') | KeyCode::Char('M') => app.toggle_mode(),
                        KeyCode::Char('a') | KeyCode::Char('A') => app.cycle_algorithm(),
                        KeyCode::Char('c') | KeyCode::Char('C') => app.clear_data(),
                        KeyCode::Char('t') | KeyCode::Char('T') => app.train_models(),
                        KeyCode::Char('e') | KeyCode::Char('E') => app.export_csv(),
                        _ => {}
                    }
                }
            }
        }

        // Advance simulation and telemetry tick
        app.tick();
    }

    // 4. Restore terminal cleanly on exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
