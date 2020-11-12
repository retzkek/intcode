use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};

use super::Program;

pub fn debug(prog: &mut Program) -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(40),
                    Constraint::Percentage(10)
                ].as_ref()
            )
            .split(f.size());
        let block = Block::default()
             .title("Code")
             .borders(Borders::ALL);
        f.render_widget(block, chunks[0]);

        let block = Block::default()
             .title("Block 2")
             .borders(Borders::ALL);
        f.render_widget(block, chunks[1]);
    })
}
