//! Log Streamer - Real-time log aggregation and filtering
//! Showcases: Text Input, List virtualization (simulated), Derived State, Master-Detail view, Dynamic filtering

use rat_nexus::prelude::*;
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment},
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType, Wrap, ListState},
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
            self.logs.remove(0); // Keep buffer capped
            // Indices shift invalidation is complex, so simpler to just recalc filter
            self.recalc_filter();
        } else {
            // Optimization: check if new log matches filter, if so add to filtered_indices
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
        
        // Clamp selection
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

        // Observe
        self.tasks.track(cx.observe(&state));

        // Generator Task
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
                        service, 
                        rng.gen::<u64>(), 
                        rng.gen_range(0..16),
                        methods[rng.gen_range(0..methods.len())],
                        rng.gen_range(5..500)
                     );
                     
                     let log = LogEntry {
                         id: id_counter,
                         timestamp: elapsed,
                         level,
                         service,
                         message: msg,
                         details,
                     };

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
        
        canvas(move |frame, area| {
             let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header/Filter
                    Constraint::Min(0),     // Content
                    Constraint::Length(3),  // Footer
                ])
                .split(area);

            // === 1. Header / Filter ===
            let filter_style = if state_data.is_typing { 
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
            } else { 
                Style::default().fg(Color::Cyan) 
            };
            
            let filter_border_style = if state_data.is_typing {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let filter_text = if state_data.filter.is_empty() { 
                if state_data.is_typing { "" } else { "Type '/' to search..." } 
            } else { 
                &state_data.filter 
            };
            
            let header = Paragraph::new(format!(" 🔍 {}", filter_text))
                .block(Block::default()
                    .title(" Log Filter ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(filter_border_style))
                .style(filter_style);
            
            frame.render_widget(header, main_layout[0]);
            
            // Draw Cursor if typing
            if state_data.is_typing {
                let cursor_x = main_layout[0].x + 4 + state_data.filter.len() as u16;
                let cursor_y = main_layout[0].y + 1;
                // Simple cursor simulation
                frame.set_cursor_position((cursor_x, cursor_y)); 
            }

            // === 2. Content (Split List and Detail) ===
            let content_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(60), // List
                    Constraint::Percentage(40), // Detail
                ])
                .split(main_layout[1]);

            // LIST
            let items: Vec<ListItem> = state_data.filtered_indices.iter().map(|&idx| {
                if let Some(log) = state_data.logs.get(idx) {
                    let time_str = format!("{:>6.2}s", log.timestamp);
                    let level_style = Style::default().fg(log.level.color());
                    
                    Line::from(vec![
                        Span::styled(format!(" {} ", time_str), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!(" {} ", log.level.as_str()), level_style.add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" {:<12} ", log.service), Style::default().fg(Color::Blue)),
                        Span::raw(&log.message),
                    ]).into()
                } else {
                    ListItem::new("Invalid Log Index")
                }
            }).collect();

            let logs_block = Block::default()
                .title(" Live Logs ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if state_data.is_typing { Color::DarkGray } else { Color::White }));

            let mut list_state = ListState::default();
            
            // Adjust scroll to keep selection visible
            if !state_data.filtered_indices.is_empty() {
                list_state.select(Some(state_data.selected_index));
            }

            let list = List::new(items)
                .block(logs_block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
                
            frame.render_stateful_widget(list, content_layout[0], &mut list_state);


            // DETAIL
            let selected_log = state_data.filtered_indices.get(state_data.selected_index)
                .and_then(|&idx| state_data.logs.get(idx));
                
            let detail_text = if let Some(log) = selected_log {
                vec![
                    Line::from(vec![Span::styled("Log ID: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{}", log.id))]),
                    Line::from(vec![Span::styled("Service: ", Style::default().fg(Color::Cyan)), Span::raw(&log.service)]),
                    Line::from(vec![Span::styled("Time:    ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.3}s", log.timestamp))]),
                    Line::from(vec![Span::styled("Level:   ", Style::default().fg(Color::Cyan)), Span::styled(log.level.as_str(), Style::default().fg(log.level.color()))]),
                    Line::from(""),
                    Line::from(Span::styled("Payload:", Style::default().fg(Color::Yellow))),
                    Line::from(log.details.clone()),
                ]
            } else {
                vec![Line::from("No log selected...")]
            };

            let detail = Paragraph::new(detail_text)
                .wrap(Wrap { trim: false })
                .block(Block::default()
                    .title(" Log Details ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick));
            
            frame.render_widget(detail, content_layout[1]);


            // === 3. Footer ===
            let (n_info, n_warn, n_err) = state_data.stats;
            let status_space = if state_data.paused { "PAUSED (Space to Resume)" } else { "STREAMING (Space to Pause)" };
            let auto_scroll_status = if state_data.auto_scroll { "Auto-Scroll: ON (A)" } else { "Auto-Scroll: OFF (A)" };
            
            let footer_content = vec![
                Span::styled(format!(" Total: {} ", state_data.logs.len()), Style::default().fg(Color::White)),
                Span::raw(" | "),
                Span::styled(format!(" Info: {} ", n_info), Style::default().fg(Color::Green)),
                Span::styled(format!(" Warn: {} ", n_warn), Style::default().fg(Color::Yellow)),
                Span::styled(format!(" Err: {} ", n_err), Style::default().fg(Color::Red)),
                Span::raw(" │ "),
                Span::styled(status_space, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" │ "),
                Span::styled(auto_scroll_status, Style::default()),
                Span::raw(" │ M: Menu Q: Quit"),
            ];

            let footer = Paragraph::new(Line::from(footer_content))
                .alignment(Alignment::Center)
                .style(Style::default().bg(Color::DarkGray))
                .block(Block::default().borders(Borders::TOP));
            
            frame.render_widget(footer, main_layout[2]);
        })
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        let is_typing = self.state.read(|s| s.is_typing).unwrap_or(false);

        match event {
            Event::Key(key) => {
                if is_typing {
                    // Typing Mode
                    match key.code {
                         KeyCode::Esc | KeyCode::Enter => {
                             let _ = self.state.update(|s| s.is_typing = false);
                             None
                         }
                         KeyCode::Backspace => {
                             let _ = self.state.update(|s| {
                                 s.filter.pop();
                                 s.recalc_filter();
                             });
                             None
                         }
                         KeyCode::Char(c) => {
                             let _ = self.state.update(|s| {
                                 s.filter.push(c);
                                 s.recalc_filter();
                             });
                             None
                         }
                         _ => None,
                    }
                } else {
                    // Navigation Mode
                    match key.code {
                        KeyCode::Char('q') => Some(Action::Quit),
                        KeyCode::Char('m') => Some(Action::Navigate("menu".to_string())),
                        KeyCode::Char('/') => {
                            let _ = self.state.update(|s| {
                                s.is_typing = true;
                                s.auto_scroll = false; // Disable auto scroll when searching
                            });
                            None
                        },
                        KeyCode::Char(' ') => {
                            let _ = self.state.update(|s| s.paused = !s.paused);
                            None
                        }
                        KeyCode::Char('a') => {
                            let _ = self.state.update(|s| { 
                                s.auto_scroll = !s.auto_scroll;
                                // Jump to bottom if enabling
                                if s.auto_scroll && !s.filtered_indices.is_empty() {
                                    s.selected_index = s.filtered_indices.len() - 1;
                                }
                            });
                            None
                        }
                        KeyCode::Char('c') => {
                            let _ = self.state.update(|s| {
                                s.logs.clear();
                                s.filtered_indices.clear();
                                s.selected_index = 0;
                                s.stats = (0, 0, 0);
                            });
                            None
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let _ = self.state.update(|s| {
                                s.auto_scroll = false; // User manual interaction disrupts auto-scroll
                                if s.selected_index > 0 {
                                    s.selected_index -= 1;
                                }
                            });
                            None
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let _ = self.state.update(|s| {
                                s.auto_scroll = false;
                                if s.selected_index < s.filtered_indices.len().saturating_sub(1) {
                                    s.selected_index += 1;
                                } else if s.selected_index == s.filtered_indices.len().saturating_sub(1) {
                                     // if at bottom, maybe re-enable auto scroll? optional.
                                }
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
