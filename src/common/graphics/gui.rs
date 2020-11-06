use alloc::{collections::VecDeque, string::String, vec::Vec};
use log::{Level, Record};
use spin::{Mutex, Once};
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::List,
    widgets::ListItem,
    widgets::{Block, Borders, ListState},
    Terminal,
};

use super::fb::FrameBrufferDisplay;

pub struct GuiState {
    logs: VecDeque<(Level, String)>,
    logs_count: usize,
}

impl GuiState {
    pub fn new() -> GuiState {
        GuiState {
            logs: VecDeque::new(),
            logs_count: 26,
        }
    }

    pub fn add_log<'a>(&mut self, record: &Record<'a>) {
        if self.logs.len() >= self.logs_count {
            self.logs.pop_front();
        }

        let val = format!(
            "{} [{}] -- {}",
            record.level(),
            record.target(),
            record.args()
        );

        self.logs.push_back((record.level(), val));
    }
}

fn draw<'a>(
    state: &GuiState,
    terminal: &mut Terminal<FrameBrufferDisplay<'a>>,
) -> Result<(), String> {
    terminal
        .draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let log_items: Vec<ListItem> = state
                .logs
                .iter()
                .map(|record| {
                    let item = ListItem::new(record.1.clone());
                    let item = match record.0 {
                        log::Level::Error => item.style(Style::default().fg(Color::Red)),
                        log::Level::Warn => item.style(Style::default().fg(Color::Yellow)),
                        log::Level::Info => item.style(Style::default().fg(Color::White)),
                        log::Level::Debug => item.style(Style::default().fg(Color::Gray)),
                        log::Level::Trace => item.style(Style::default().fg(Color::Gray)),
                    };
                    item
                })
                .collect();
            let log_list =
                List::new(log_items).block(Block::default().title("Logs").borders(Borders::ALL));
            let mut state = ListState::default();
            f.render_stateful_widget(log_list, chunks[0], &mut state);
        })
        .unwrap();

    Ok(())
}

static GUI_STATE: Once<Mutex<GuiState>> = Once::new();
static TERMINAL: Once<Mutex<Terminal<FrameBrufferDisplay<'static>>>> = Once::new();

pub fn initialize(terminal: Terminal<FrameBrufferDisplay<'static>>) {
    GUI_STATE.call_once(|| Mutex::new(GuiState::new()));
    TERMINAL.call_once(move || Mutex::new(terminal));
}

/// A logger implementation for Gui.
pub struct GuiLogger;

impl log::Log for GuiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        let state = &mut GUI_STATE.get().unwrap().lock();
        state.add_log(record);
    }

    fn flush(&self) {
        let state = &GUI_STATE.get().unwrap().lock();
        let mut terminal = &mut TERMINAL.get().unwrap().lock();
        draw(&state, &mut terminal).unwrap();
    }
}
