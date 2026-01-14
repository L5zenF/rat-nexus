//! Timer Demo - Stopwatch with lap times
//! Showcases: Entity state, spawn_task, TaskTracker, async updates

use rat_nexus::prelude::*;
use ratatui::{
    layout::Alignment,
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType},
    style::{Style, Color, Modifier},
    text::{Line, Span},
};
use crossterm::event::KeyCode;

#[derive(Clone, Default)]
pub struct TimerState {
    pub elapsed_ms: u64,
    pub running: bool,
    pub laps: Vec<u64>,
}

#[derive(Default)]
pub struct TimerPage {
    state: Entity<TimerState>,
    tasks: TaskTracker,
}

impl Component for TimerPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // Initialize state entity
        let state = cx.new_entity(TimerState::default());
        self.state = Entity::clone(&state);

        // Observe for re-renders
        self.tasks.track(cx.observe(&self.state));

        let handle = cx.spawn_detached_task(move |_app| async move {
            loop {
                let running = state.read(|s| s.running).unwrap_or(false);
                if running {
                    let _ = state.update(|s| s.elapsed_ms += 10);
                    // app.refresh(); // redundant with observer
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    fn render(&mut self, _cx: &mut Context<Self>) -> impl IntoElement + 'static {
        let state_data = self.state.read(|s| s.clone()).unwrap_or_default();

        // Timer display
        let time = format_time(state_data.elapsed_ms);
        let color = if state_data.running { Color::Green } else { Color::Yellow };

        let timer_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(time, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ]).alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if state_data.running { "  RUNNING  " } else { "  STOPPED  " },
                    Style::default().fg(Color::Black).bg(color)
                ),
            ]).alignment(Alignment::Center),
        ];

        let header = div()
            .h(9)
            .border_all()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color))
            .title(" Stopwatch ")
            .child(widget(Paragraph::new(timer_lines)));

        // Lap times
        let lap_items: Vec<ListItem> = state_data.laps.iter().enumerate().rev()
            .map(|(i, &ms)| {
                ListItem::new(format!("  Lap {:02}  {}  ", i + 1, format_time(ms)))
                    .style(Style::default().fg(Color::Cyan))
            })
            .collect();

        let lap_list = widget(List::new(lap_items)
            .block(Block::default()
                .title(format!(" Laps ({}) ", state_data.laps.len()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))))
            .h_full();

        // Footer
        let footer = div()
            .h(3)
            .bg(color)
            .fg(Color::Black)
            .child(text(" SPACE Start/Stop │ L Lap │ R Reset │ M Menu │ Q Quit ").align_center());

        // Layout
        div()
            .flex_col()
            .h_full()
            .child(header)
            .child(div().flex().child(lap_list))
            .child(footer)
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => Some(Action::Navigate("menu".to_string())),
                KeyCode::Char(' ') => {
                    let _ = self.state.update(|s| s.running = !s.running);
                    None
                }
                KeyCode::Char('l') => {
                    let _ = self.state.update(|s| {
                        if s.running || s.elapsed_ms > 0 {
                            s.laps.push(s.elapsed_ms);
                        }
                    });
                    None
                }
                KeyCode::Char('r') => {
                    let _ = self.state.update(|s| {
                        s.elapsed_ms = 0;
                        s.running = false;
                        s.laps.clear();
                    });
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn format_time(ms: u64) -> String {
    let mins = ms / 60000;
    let secs = (ms % 60000) / 1000;
    let centis = (ms % 1000) / 10;
    format!("{:02}:{:02}.{:02}", mins, secs, centis)
}
