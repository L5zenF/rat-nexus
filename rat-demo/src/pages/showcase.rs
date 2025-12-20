//! Widget Showcase - Demonstrates various ratatui widgets.
//!
//! This page showcases:
//! - Tabs widget for navigation
//! - Table widget with selection
//! - BarChart for data visualization
//! - List with scrolling
//! - Sparkline for inline charts
//! - Gauge variants

use rat_nexus::{Component, Context, EventContext, Event, Action, Entity, AppContext};
use crate::model::{AppState, ShowcaseState};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    widgets::{
        Block, Borders, Paragraph, Tabs, Table, Row, Cell, BarChart, List, ListItem,
        Sparkline, Gauge, LineGauge, BorderType,
    },
    style::{Style, Color, Modifier},
    text::{Line, Span},
    symbols,
};
use crossterm::event::KeyCode;

pub struct ShowcasePage {
    app_state: Entity<AppState>,
    state: Entity<ShowcaseState>,
}

impl ShowcasePage {
    pub fn new(app_state: Entity<AppState>, cx: &AppContext) -> Self {
        Self {
            app_state,
            state: cx.new_entity(ShowcaseState::default()),
        }
    }
}

impl Component for ShowcasePage {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        cx.subscribe(&self.state);
        cx.subscribe(&self.app_state);

        let state = self.state.read(|s| s.clone()).unwrap_or_default();
        let app = self.app_state.read(|s| s.clone()).unwrap_or_default();
        let theme_color = app.theme.color();

        let area = frame.area();

        // Main layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header with tabs
        let tab_titles = vec!["Table", "Charts", "Lists", "Gauges"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default()
                .title(" 📦 Widget Showcase ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .select(state.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(theme_color).add_modifier(Modifier::BOLD))
            .divider(symbols::line::VERTICAL);
        frame.render_widget(tabs, main_layout[0]);

        // Render content based on selected tab
        match state.selected_tab {
            0 => self.render_table_tab(frame, main_layout[1], &state, theme_color),
            1 => self.render_charts_tab(frame, main_layout[1], &state, theme_color),
            2 => self.render_lists_tab(frame, main_layout[1], &state, theme_color),
            3 => self.render_gauges_tab(frame, main_layout[1], theme_color),
            _ => {}
        }

        // Footer
        let footer = Paragraph::new(" 1-4 Tabs │ ←/→/Tab Switch │ ↑/↓ Navigate │ Enter Select │ T Theme │ M Menu │ Q Quit ")
            .style(Style::default().bg(theme_color).fg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_layout[2]);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => return Some(Action::Navigate("menu".to_string())),
                KeyCode::Char('t') => {
                    let _ = self.app_state.update(|s| s.theme = s.theme.next());
                }
                // Number keys for quick tab switching
                KeyCode::Char('1') => {
                    let _ = self.state.update(|s| s.selected_tab = 0);
                }
                KeyCode::Char('2') => {
                    let _ = self.state.update(|s| s.selected_tab = 1);
                }
                KeyCode::Char('3') => {
                    let _ = self.state.update(|s| s.selected_tab = 2);
                }
                KeyCode::Char('4') => {
                    let _ = self.state.update(|s| s.selected_tab = 3);
                }
                // Tab navigation
                KeyCode::Left | KeyCode::Char('h') => {
                    let _ = self.state.update(|s| {
                        if s.selected_tab > 0 {
                            s.selected_tab -= 1;
                        } else {
                            s.selected_tab = 3;
                        }
                    });
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let _ = self.state.update(|s| {
                        s.selected_tab = (s.selected_tab + 1) % 4;
                    });
                }
                KeyCode::Tab => {
                    let _ = self.state.update(|s| {
                        s.selected_tab = (s.selected_tab + 1) % 4;
                    });
                }
                KeyCode::BackTab => {
                    let _ = self.state.update(|s| {
                        if s.selected_tab > 0 {
                            s.selected_tab -= 1;
                        } else {
                            s.selected_tab = 3;
                        }
                    });
                }
                // Item navigation within tabs
                KeyCode::Up | KeyCode::Char('k') => {
                    let _ = self.state.update(|s| {
                        if s.table_selected > 0 {
                            s.table_selected -= 1;
                        }
                    });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let _ = self.state.update(|s| {
                        s.table_selected += 1;
                    });
                }
                KeyCode::Home => {
                    let _ = self.state.update(|s| s.table_selected = 0);
                }
                KeyCode::End => {
                    let _ = self.state.update(|s| s.table_selected = 100); // Will be clamped
                }
                KeyCode::Enter => {
                    // Could trigger an action based on selected item
                    // For now, cycle theme as feedback
                    let _ = self.app_state.update(|s| s.theme = s.theme.next());
                }
                _ => {}
            },
            Event::Mouse(mouse) => {
                use crossterm::event::{MouseEventKind, MouseButton};
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        let _ = self.state.update(|s| {
                            if s.table_selected > 0 {
                                s.table_selected -= 1;
                            }
                        });
                    }
                    MouseEventKind::ScrollDown => {
                        let _ = self.state.update(|s| {
                            s.table_selected += 1;
                        });
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Left click cycles tabs
                        let _ = self.state.update(|s| {
                            s.selected_tab = (s.selected_tab + 1) % 4;
                        });
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        // Right click cycles theme
                        let _ = self.app_state.update(|s| s.theme = s.theme.next());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        None
    }
}

impl ShowcasePage {
    fn render_table_tab(&self, frame: &mut ratatui::Frame, area: Rect, state: &ShowcaseState, theme_color: Color) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .margin(1)
            .split(area);

        // Table data - Framework features
        let rows = vec![
            ("Entity<T>", "Reactive", "State container with change notifications"),
            ("Component", "Lifecycle", "on_mount, on_enter, on_exit, on_shutdown"),
            ("Context<V>", "GPUI-style", "entity_id(), entity(), weak_entity()"),
            ("TaskTracker", "Async", "Track and cancel spawned tasks"),
            ("Router", "Navigation", "Type-safe route management"),
            ("EntityId", "Identity", "Unique identifier for each entity"),
            ("WeakEntity", "Reference", "Non-owning entity reference"),
            ("AppContext", "Global", "Application-wide context access"),
        ];

        let selected = state.table_selected.min(rows.len().saturating_sub(1));

        let table_rows: Vec<Row> = rows
            .iter()
            .enumerate()
            .map(|(i, (name, category, desc))| {
                let style = if i == selected {
                    Style::default().fg(Color::Black).bg(theme_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(*name).style(Style::default().fg(theme_color)),
                    Cell::from(*category),
                    Cell::from(*desc),
                ]).style(style)
            })
            .collect();

        let table = Table::new(
            table_rows,
            [Constraint::Length(12), Constraint::Length(10), Constraint::Min(20)],
        )
        .header(
            Row::new(vec!["Name", "Type", "Description"])
                .style(Style::default().fg(theme_color).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(Block::default()
            .title(" Framework Features ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme_color)));

        frame.render_widget(table, chunks[0]);

        // Feature detail panel
        let detail_text = if selected < rows.len() {
            let (name, category, desc) = rows[selected];
            vec![
                Line::from(vec![
                    Span::styled("Feature: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(name, Style::default().fg(theme_color).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Category: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(category, Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description:", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(format!("  {}", desc)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Usage example:", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(""),
                Line::styled(format!("  let {} = ...", name.to_lowercase().replace("<t>", "")),
                    Style::default().fg(Color::Green)),
            ]
        } else {
            vec![Line::from("Select an item")]
        };

        let detail = Paragraph::new(detail_text)
            .block(Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)));

        frame.render_widget(detail, chunks[1]);
    }

    fn render_charts_tab(&self, frame: &mut ratatui::Frame, area: Rect, state: &ShowcaseState, theme_color: Color) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .split(area);

        // BarChart
        let bar_chart = BarChart::default()
            .block(Block::default()
                .title(" BarChart - Weekly Activity ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .bar_width(7)
            .bar_gap(2)
            .group_gap(3)
            .bar_style(Style::default().fg(theme_color))
            .value_style(Style::default().fg(Color::Black).bg(theme_color))
            .label_style(Style::default().fg(Color::White))
            .data(&state.bar_data);

        frame.render_widget(bar_chart, chunks[0]);

        // Sparklines row
        let spark_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Sparkline 1
        let spark_data: Vec<u64> = state.chart_data.iter().map(|(_, y)| *y as u64).collect();
        let sparkline1 = Sparkline::default()
            .block(Block::default()
                .title(" Sparkline - Sine Wave ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .data(&spark_data)
            .style(Style::default().fg(theme_color));

        frame.render_widget(sparkline1, spark_chunks[0]);

        // Sparkline 2 - Different data
        let spark_data2: Vec<u64> = (0..50).map(|i| ((i as f64 * 0.3).cos() * 40.0 + 50.0) as u64).collect();
        let sparkline2 = Sparkline::default()
            .block(Block::default()
                .title(" Sparkline - Cosine Wave ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .data(&spark_data2)
            .style(Style::default().fg(Color::Yellow));

        frame.render_widget(sparkline2, spark_chunks[1]);
    }

    fn render_lists_tab(&self, frame: &mut ratatui::Frame, area: Rect, state: &ShowcaseState, theme_color: Color) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .split(area);

        // List with selection
        let items: Vec<ListItem> = state.list_items.iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == state.table_selected % state.list_items.len() { "▶ " } else { "  " };
                let style = if i == state.table_selected % state.list_items.len() {
                    Style::default().fg(theme_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title(" List - Framework Components ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        frame.render_widget(list, chunks[0]);

        // Info panel with styled text
        let info_lines = vec![
            Line::from(vec![
                Span::styled("🦀 ", Style::default()),
                Span::styled("Rat-Nexus", Style::default().fg(theme_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("A modern ", Style::default().fg(Color::White)),
                Span::styled("reactive", Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)),
                Span::styled(" TUI framework", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::styled("Features:", Style::default().fg(Color::DarkGray)),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::raw("GPUI-inspired architecture"),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::raw("Reactive state management"),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::raw("Component lifecycle hooks"),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::raw("Async task management"),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::raw("Type-safe routing"),
            ]),
            Line::from(""),
            Line::styled("Built with Ratatui", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ];

        let info = Paragraph::new(info_lines)
            .block(Block::default()
                .title(" Styled Text Demo ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .alignment(Alignment::Left);

        frame.render_widget(info, chunks[1]);
    }

    fn render_gauges_tab(&self, frame: &mut ratatui::Frame, area: Rect, theme_color: Color) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(area);

        // Regular Gauge
        let gauge1 = Gauge::default()
            .block(Block::default().title(" Gauge - CPU Usage ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(theme_color))
            .percent(65)
            .label("65%");
        frame.render_widget(gauge1, chunks[0]);

        // Gauge with different ratio
        let gauge2 = Gauge::default()
            .block(Block::default().title(" Gauge - Memory ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(45)
            .label("45% (4.5/10 GB)");
        frame.render_widget(gauge2, chunks[1]);

        // LineGauge
        let line_gauge1 = LineGauge::default()
            .block(Block::default().title(" LineGauge - Download Progress ").borders(Borders::ALL))
            .filled_style(Style::default().fg(theme_color))
            .line_set(symbols::line::THICK)
            .ratio(0.78);
        frame.render_widget(line_gauge1, chunks[2]);

        // LineGauge with different style
        let line_gauge2 = LineGauge::default()
            .block(Block::default().title(" LineGauge - Build Progress ").borders(Borders::ALL))
            .filled_style(Style::default().fg(Color::Yellow))
            .line_set(symbols::line::DOUBLE)
            .ratio(0.35);
        frame.render_widget(line_gauge2, chunks[3]);

        // Unicode gauge
        let gauge3 = Gauge::default()
            .block(Block::default().title(" Unicode Gauge - Disk I/O ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(82)
            .use_unicode(true)
            .label("82% throughput");
        frame.render_widget(gauge3, chunks[4]);

        // Info text
        let info = Paragraph::new(vec![
            Line::from(""),
            Line::styled("  Gauge widgets are perfect for showing progress and metrics.", Style::default().fg(Color::DarkGray)),
            Line::from(""),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::styled("Gauge", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Standard progress bar"),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::styled("LineGauge", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Thin line-style progress"),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme_color)),
                Span::styled("use_unicode(true)", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Smoother rendering"),
            ]),
        ])
        .block(Block::default()
            .title(" Gauge Variants ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme_color)));
        frame.render_widget(info, chunks[5]);
    }
}
