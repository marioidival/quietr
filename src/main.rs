use std::{
    io::{stdout, Result},
    time::Duration,
};

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use pdf::pdf2text;
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

pub enum Mode {
    Word,
    Phrase,
}

pub struct App {
    pub is_running: bool,
}

mod pdf;

const DURATION: Duration = Duration::from_millis(60 * 1000 / 300);

fn main() -> Result<()> {
    let file_path = std::env::args().nth(1).expect("No file path provided");

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = App { is_running: true };
    // HACK: for while this mode could be not mutable
    let mode = Mode::Phrase;

    let tree = pdf2text(file_path)?;

    'out: loop {
        match mode {
            Mode::Phrase => {
                for (page, lines) in tree.text.iter().skip(128) {
                    for line in lines.iter() {
                        for phrase in line.split_inclusive('.') {
                            terminal.draw(|f| {
                                let text = vec![
                                    Line::from(vec![
                                        Span::raw("Page Number: "),
                                        Span::styled(
                                            format!("{}", page),
                                            Style::default().fg(Color::White),
                                        ),
                                    ]),
                                    Line::from(phrase.to_string().bold()),
                                ];

                                let size = f.size();
                                f.render_widget(
                                    Paragraph::new(text)
                                        .centered()
                                        .white()
                                        .on_blue()
                                        .wrap(Wrap { trim: true }),
                                    size,
                                );
                            })?;

                            app.is_running = process_input()?;

                            if !app.is_running {
                                break 'out;
                            }

                            std::thread::sleep(DURATION * phrase.split_whitespace().count() as u32);
                        }
                    }
                }
            }
            Mode::Word => {
                for (page, lines) in tree.text.iter().skip(125) {
                    for line in lines.iter() {
                        for word in line.split_whitespace() {
                            terminal.draw(|f| {
                                let text = vec![
                                    Line::from(vec![
                                        Span::raw("Page Number: "),
                                        Span::styled(
                                            format!("{}", page),
                                            Style::default().fg(Color::White),
                                        ),
                                    ]),
                                    Line::from(word.to_string().bold()),
                                ];

                                let size = f.size();
                                f.render_widget(
                                    Paragraph::new(text).centered().white().on_blue(),
                                    size,
                                );
                            })?;

                            app.is_running = process_input()?;

                            if !app.is_running {
                                break 'out;
                            }

                            std::thread::sleep(DURATION);
                        }
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn process_input() -> Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
