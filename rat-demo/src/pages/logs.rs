//! Log Streamer - Real-time log aggregation and filtering
//! Showcases: Text Input, List virtualization (simulated), Derived State, Master-Detail view, Dynamic filtering
//! Now leveraging built-in component library (Div, Text, Canvas)

use rat_nexus::prelude::*;
use ratatui::{
    widgets::{Paragraph, List, ListItem, BorderType, Wrap, ListState},
    style::{Style, Color, Modifier},
    text::{Line, Span},
};
use crossterm::event::KeyCode;
use std::time::{SystemTime};
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            LogLevel::Info => Color::Green,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Error => Color::Red,
            LogLevel::Debug => Color::Blue,
            LogLevel::Trace => Color::DarkGray,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERR ",
            LogLevel::Debug => "DBUG",
            LogLevel::Trace => "TRCE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: f64, // Seconds since start
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub details: String, // Simulated JSON payload
}

#[derive(Clone)]
pub struct LogState {
    pub logs: Vec<LogEntry>,
    pub filter: String,
    pub is_typing: bool,
    pub paused: bool,
    pub auto_scroll: bool,
    pub selected_index: usize, // Index within filtered results
    pub filtered_indices: Vec<usize>, // Indicies of logs that match filter
    pub stats: (usize, usize, usize), // Info, Warn, Error
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            filter: String::new(),
            is_typing: false,
            paused: false,
            auto_scroll: true,
            selected_index: 0,
            filtered_indices: Vec::new(),
            stats: (0, 0, 0),
        }
    }
}

impl LogState {
    fn add_log(&mut self, log: LogEntry) {
        match log.level {
            LogLevel::Info => self.stats.0 += 1,
            LogLevel::Warn => self.stats.1 += 1,
            LogLevel::Error => self.stats.2 += 1,
            _ => {}
        }
        
        self.logs.push(log);
        if self.logs.len() > 1000 {
            self.logs.remove(0); 
            self.recalc_filter();
        } else {
            let matches = self.filter.is_empty() || 
                          self.logs.last().unwrap().message.to_lowercase().contains(&self.filter.to_lowercase()) ||
                          self.logs.last().unwrap().service.to_lowercase().contains(&self.filter.to_lowercase());
            
            if matches {
                self.filtered_indices.push(self.logs.len() - 1);
            }
        }

        if self.auto_scroll && !self.filtered_indices.is_empty() {
             self.selected_index = self.filtered_indices.len().saturating_sub(1);
        }
    }

    fn recalc_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.logs.len()).collect();
        } else {
            let query = self.filter.to_lowercase();
            self.filtered_indices = self.logs.iter().enumerate()
                .filter(|(_, log)| {
                    log.message.to_lowercase().contains(&query) || 
                    log.service.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        
        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
        }
    }
}

pub struct LogPage {
    state: Entity<LogState>,
    tasks: TaskTracker,
}

impl Default for LogPage {
    fn default() -> Self {
        Self {
            state: Entity::default(),
            tasks: TaskTracker::new(),
        }
    }
}

impl Component for LogPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        let state = cx.new_entity(LogState::default());
        self.state = Entity::clone(&state);

        self.tasks.track(cx.observe(&state));

        let bg_state = state.downgrade();
        let handle = cx.spawn_detached_task(move |_| async move {
             use rand::SeedableRng;
             let mut rng = rand::rngs::StdRng::from_entropy();
             let start_time = SystemTime::now();
             let mut id_counter = 0;

             let services = ["auth-svc", "db-shard-01", "api-gateway", "payment-proc", "front-nginx", "analytics", "search-idx"];
             let messages = [
                 "Connection established", "Query executed in 23ms", "Cache miss", 
                 "User login successful", "Payment processed", "Packet dropped", 
                 "Retrying connection...", "Buffer overflow warning", "Index optimization triggered",
                 "Garbage collection started", "Health check passed", "Rate limit exceeded"
             ];
             let methods = ["GET", "POST", "PUT", "DELETE"];

             loop {
                 let should_add = if let Some(s) = bg_state.upgrade() {
                     s.read(|st| !st.paused).unwrap_or(false)
                 } else { false };

                 if should_add {
                     let elapsed = SystemTime::now().duration_since(start_time).unwrap_or_default().as_secs_f64();
                     id_counter += 1;
                     
                     let level_rnd = rng.gen_range(0..100);
                     let level = if level_rnd < 60 { LogLevel::Info }
                                 else if level_rnd < 85 { LogLevel::Warn }
                                 else if level_rnd < 95 { LogLevel::Error }
                                 else { LogLevel::Debug };

                     let service = services[rng.gen_range(0..services.len())].to_string();
                     let msg_base = messages[rng.gen_range(0..messages.len())];
                     let msg = match level {
                         LogLevel::Error => format!("Failed to complete: {}", msg_base),
                         LogLevel::Debug => format!("[TRACE] {}", msg_base),
                         _ => msg_base.to_string(),
                     };
                     
                     let details = format!("{{\n  \"id\": \"Log-{id_counter}\",\n  \"svc\": \"{}\",\n  \"trace\": \"{:016x}\",\n  \"shard\": {},\n  \"method\": \"{}\",\n  \"latency_ms\": {}\n}}", 
                        service, rng.gen::<u64>(), rng.gen_range(0..16), methods[rng.gen_range(0..methods.len())], rng.gen_range(5..500)
                     );
                     
                     let log = LogEntry { id: id_counter, timestamp: elapsed, level, service, message: msg, details };
                     if let Some(s) = bg_state.upgrade() {
                         let _ = s.update(|st| st.add_log(log));
                     }
                 }
                 let delay = if should_add { rng.gen_range(200..1500) } else { 500 };
                 tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
             }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    fn render(&mut self, _cx: &mut Context<Self>) -> impl IntoElement + 'static {
        let state_data = self.state.read(|s| s.clone()).unwrap_or_default();
        
        // --- 1. Header (Div + Canvas for input simulation with cursor) ---
        let state_data_c1 = state_data.clone();
        let header = div()
            .h(3)
            .border_all()
            .border_type(BorderType::Rounded)
            .title(" Log Filter ")
            .fg(if state_data.is_typing { Color::Yellow } else { Color::DarkGray })
            .px(1)
            .child(
                canvas(move |frame, area| {
                    let text = if state_data_c1.filter.is_empty() { 
                        if state_data_c1.is_typing { "" } else { "Type '/' to search..." } 
                    } else { 
                        &state_data_c1.filter 
                    };
                    let style = if state_data_c1.is_typing { 
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
                    } else { 
                        Style::default().fg(Color::Cyan) 
                    };
                    frame.render_widget(Paragraph::new(format!(" 🔍 {}", text)).style(style), area);
                    if state_data_c1.is_typing {
                        frame.set_cursor_position((area.x + 3 + state_data_c1.filter.len() as u16, area.y));
                    }
                })
            );

        // --- 2. Content Row (Div FlexBox) ---
        let state_data_c2 = state_data.clone();
        let list_view = div()
            .w_percent(60)
            .border_all()
            .title(" Live Logs ")
            .child(
                canvas(move |frame, area| {
                    let items: Vec<ListItem> = state_data_c2.filtered_indices.iter().map(|&idx| {
                        if let Some(log) = state_data_c2.logs.get(idx) {
                            let time_str = format!("{:>6.2}s", log.timestamp);
                            Line::from(vec![
                                Span::styled(format!(" {} ", time_str), Style::default().fg(Color::DarkGray)),
                                Span::styled(format!(" {} ", log.level.as_str()), Style::default().fg(log.level.color()).add_modifier(Modifier::BOLD)),
                                Span::styled(format!(" {:<12} ", log.service), Style::default().fg(Color::Blue)),
                                Span::raw(log.message.clone()),
                            ]).into()
                        } else { ListItem::new("error") }
                    }).collect();

                    let mut list_state = ListState::default();
                    if !state_data_c2.filtered_indices.is_empty() {
                        list_state.select(Some(state_data_c2.selected_index));
                    }
                    let list = List::new(items)
                        .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> ");
                    frame.render_stateful_widget(list, area, &mut list_state);
                })
            );

        let selected_log = state_data.filtered_indices.get(state_data.selected_index)
            .and_then(|&idx| state_data.logs.get(idx)).cloned();

        let detail_element = if let Some(log) = selected_log {
            let detail_text = vec![
                Line::from(vec![Span::styled("Log ID:    ", Style::default().fg(Color::Cyan)), Span::raw(format!("{}", log.id))]),
                Line::from(vec![Span::styled("Service:   ", Style::default().fg(Color::Cyan)), Span::raw(log.service.clone())]),
                Line::from(vec![Span::styled("Time:      ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.3}s", log.timestamp))]),
                Line::from(vec![Span::styled("Level:     ", Style::default().fg(Color::Cyan)), Span::styled(log.level.as_str(), Style::default().fg(log.level.color()))]),
                Line::from(""),
                Line::from(Span::styled("Payload:", Style::default().fg(Color::Yellow))),
                Line::from(log.details.clone()),
            ];
            widget(Paragraph::new(detail_text).wrap(Wrap { trim: false }))
        } else {
            widget(Paragraph::new("No selection"))
        };

        let content = div()
            .flex_row()
            .child(list_view)
            .child(
                // Detail View
                div()
                    .w_percent(40)
                    .border_all()
                    .title(" Log Details ")
                    .p(1)
                    .child(detail_element)
            );

        // --- 3. Footer (Rich Div FlexRow with text bits) ---
        let (n_info, n_warn, n_err) = state_data.stats;
        let status_desc = if state_data.paused { "PAUSED" } else { "STREAMING" };
        let auto_scroll_desc = if state_data.auto_scroll { "Auto-Scroll: ON" } else { "Auto-Scroll: OFF" };

        let footer = div()
            .h(1)
            .flex_row()
            .bg(Color::Rgb(40, 40, 40))
            .child(text(format!(" Total: {} ", state_data.logs.len())).fg(Color::White))
            .child(text(" | "))
            .child(text(format!(" Info: {} ", n_info)).fg(Color::Green))
            .child(text(format!(" Warn: {} ", n_warn)).fg(Color::Yellow))
            .child(text(format!(" Error: {} ", n_err)).fg(Color::Red))
            .child(div().flex()) // Spacer
            .child(text(format!(" {} ", status_desc)).bold().fg(if state_data.paused { Color::Yellow } else { Color::Green }))
            .child(text(" | "))
            .child(text(format!(" {} ", auto_scroll_desc)).fg(Color::Cyan))
            .child(text(" | M: Menu Q: Quit ").fg(Color::DarkGray));

        // --- Final Layout Assembly ---
        div()
            .flex_col()
            .h_full()
            .child(header)
            .child(content.flex()) // Content takes remaining space
            .child(footer)
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        let is_typing = self.state.read(|s| s.is_typing).unwrap_or(false);

        match event {
            Event::Key(key) => {
                if is_typing {
                    match key.code {
                         KeyCode::Esc | KeyCode::Enter => { let _ = self.state.update(|s| s.is_typing = false); None }
                         KeyCode::Backspace => { let _ = self.state.update(|s| { s.filter.pop(); s.recalc_filter(); }); None }
                         KeyCode::Char(c) => { let _ = self.state.update(|s| { s.filter.push(c); s.recalc_filter(); }); None }
                         _ => None,
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => Some(Action::Quit),
                        KeyCode::Char('m') => Some(Action::Navigate("menu".to_string())),
                        KeyCode::Char('/') => { 
                            let _ = self.state.update(|s| { s.is_typing = true; s.auto_scroll = false; }); 
                            None 
                        },
                        KeyCode::Char(' ') => { let _ = self.state.update(|s| s.paused = !s.paused); None }
                        KeyCode::Char('a') => {
                            let _ = self.state.update(|s| { 
                                s.auto_scroll = !s.auto_scroll;
                                if s.auto_scroll && !s.filtered_indices.is_empty() {
                                    s.selected_index = s.filtered_indices.len() - 1;
                                }
                            });
                            None
                        }
                        KeyCode::Char('c') => {
                            let _ = self.state.update(|s| {
                                s.logs.clear(); s.filtered_indices.clear(); s.selected_index = 0; s.stats = (0, 0, 0);
                            });
                            None
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let _ = self.state.update(|s| {
                                s.auto_scroll = false;
                                if s.selected_index > 0 { s.selected_index -= 1; }
                            });
                            None
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let _ = self.state.update(|s| {
                                s.auto_scroll = false;
                                if s.selected_index < s.filtered_indices.len().saturating_sub(1) { s.selected_index += 1; }
                            });
                            None
                        }
                        _ => None,
                    }
                }
            },
            _ => None,
        }
    }
}
