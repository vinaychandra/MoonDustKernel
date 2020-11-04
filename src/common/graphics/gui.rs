use alloc::{collections::VecDeque, string::String, vec::Vec};
use log::Record;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::List,
    widgets::ListItem,
    widgets::{Block, Borders, ListState},
    Terminal,
};

use super::fb::FrameBrufferDisplay;

pub struct GuiState<'a> {
    logs: VecDeque<Record<'a>>,
    logs_count: usize,
}

impl<'a> GuiState<'a> {
    pub fn new() -> GuiState<'a> {
        GuiState {
            logs: VecDeque::with_capacity(20),
            logs_count: 26,
        }
    }

    pub fn add_log(&mut self, record: Record<'a>) {
        if self.logs.len() >= self.logs_count {
            self.logs.pop_front();
        }

        self.logs.push_back(record);
    }
}

pub fn draw<'a>(
    state: &GuiState,
    mut terminal: Terminal<FrameBrufferDisplay<'a>>,
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
                    let val = format!(
                        "{} [{}] -- {}",
                        record.level(),
                        record.target(),
                        record.args()
                    );
                    let item = ListItem::new(val);
                    let item = match record.level() {
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
