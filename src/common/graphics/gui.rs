use alloc::string::String;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge},
    Terminal,
};

use super::fb::FrameBrufferDisplay;

struct App {
    progress1: u16,
    progress2: u16,
    progress3: f64,
    progress4: u16,
}

impl App {
    fn new() -> App {
        App {
            progress1: 0,
            progress2: 20,
            progress3: 0.33,
            progress4: 0,
        }
    }

    fn _update(&mut self) {
        self.progress1 += 5;
        if self.progress1 > 100 {
            self.progress1 = 0;
        }
        self.progress2 += 10;
        if self.progress2 > 100 {
            self.progress2 = 0;
        }
        self.progress3 += 0.001;
        if self.progress3 > 1.0 {
            self.progress3 = 0.0;
        }
        self.progress4 += 3;
        if self.progress4 > 100 {
            self.progress4 = 0;
        }
    }
}

pub fn run<'a>(mut terminal: Terminal<FrameBrufferDisplay<'a>>) -> Result<(), String> {
    // Terminal initialization

    let app = App::new();

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let gauge = Gauge::default()
                    .block(Block::default().title("Gauge1").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Yellow).bg(Color::Blue))
                    .percent(app.progress1);
                f.render_widget(gauge, chunks[0]);

                let label = format!("{}/100", app.progress2);
                let gauge = Gauge::default()
                    .block(Block::default().title("Gauge2").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Green))
                    .percent(app.progress2)
                    .label(label);
                f.render_widget(gauge, chunks[1]);

                let gauge = Gauge::default()
                    .block(Block::default().title("Gauge3").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Yellow))
                    .ratio(app.progress3);
                f.render_widget(gauge, chunks[2]);

                let label = format!("{}/100", app.progress2);
                let gauge = Gauge::default()
                    .block(Block::default().title("Gauge4"))
                    .gauge_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::ITALIC),
                    )
                    .percent(app.progress4)
                    .label(label);
                f.render_widget(gauge, chunks[3]);
            })
            .unwrap();
        break;
    }

    Ok(())
}
