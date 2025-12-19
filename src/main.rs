//! Example TUI application using the ratatui scaffolding with gpui‑style Application.

mod component;
mod router;
mod state;
mod application;

use component::Component;
use component::traits::{Event, Action};
use router::traits::Route;
use state::Entity;
use application::{Application, Context, EventContext};
use crossterm::event::KeyCode;
use crate::application::AppContext;

// Define application state (Model)
#[derive(Default, Clone)]
struct AppState {
    counter: i32,
}

// Define a menu component
struct Menu {
    selected: usize,
    options: Vec<(&'static str, Route)>,
}

impl Menu {
    fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                ("Counter Page A", "page_a".to_string()),
                ("Counter Page B", "page_b".to_string()),
                ("Exit", "exit".to_string()),
            ],
        }
    }
}

impl Component for Menu {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        use ratatui::widgets::{Block, Borders, List, ListItem};
        use ratatui::style::{Style, Modifier};

        let block = Block::default().title("Main Menu").borders(Borders::ALL);
        let inner_area = block.inner(cx.area);
        frame.render_widget(block, cx.area);

        let items: Vec<ListItem> = self.options.iter()
            .enumerate()
            .map(|(i, (label, _))| {
                let style = if i == self.selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(*label).style(style)
            })
            .collect();

        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(list, inner_area);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) if key.code == KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            Event::Key(key) if key.code == KeyCode::Down => {
                if self.selected < self.options.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                let (_, route) = &self.options[self.selected];
                if route == "exit" {
                    Some(Action::Quit)
                } else {
                    Some(Action::Navigate(route.clone()))
                }
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
struct LocalState {
    layout_horizontal: bool,
    logs: Vec<String>,
    progress: u16,
    is_working: bool,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            layout_horizontal: false,
            logs: vec!["Initialized".to_string()],
            progress: 0,
            is_working: false,
        }
    }
}

// Define a counter page component using Entity
struct CounterPage {
    title: &'static str,
    state: Entity<AppState>,
    local: Entity<LocalState>,
}

impl CounterPage {
    fn new(title: &'static str, state: Entity<AppState>, local: Entity<LocalState>) -> Self {
        Self { title, state, local }
    }

    fn log(&self, msg: String) {
        self.local.update(|s| {
            s.logs.push(msg);
            if s.logs.len() > 10 {
                s.logs.remove(0);
            }
        });
    }
}

impl Component for CounterPage {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // Subscribe to updates
        cx.subscribe(&self.state);
        cx.subscribe(&self.local);

        let counter = self.state.read(|s| s.counter);
        let local = self.local.read(|s| s.clone());
        
        use ratatui::layout::{Layout, Constraint, Direction, Rect};
        use ratatui::widgets::{Block, Borders, Paragraph, Gauge, List, ListItem};
        use ratatui::style::{Style, Color, Modifier};

        let direction = if local.layout_horizontal { Direction::Horizontal } else { Direction::Vertical };
        
        let chunks = Layout::default()
            .direction(direction)
            .margin(1)
            .constraints([
                Constraint::Percentage(25), // Counter
                Constraint::Percentage(25), // Context Info
                Constraint::Percentage(25), // Progress
                Constraint::Percentage(25), // Logs
            ])
            .split(cx.area);

        // 1. Shared State (Counter)
        let text = format!("{}\nCounter: {}", self.title, counter);
        let p1 = Paragraph::new(text)
            .block(Block::default().title("1. Global State").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(p1, chunks[0]);

        // 2. Context Info
        let area_info = format!(
            "Area: {}x{}\nOrigin: ({}, {})\n\nMode: {}", 
            cx.area.width, cx.area.height, cx.area.x, cx.area.y,
            if local.layout_horizontal { "Horizontal" } else { "Vertical" }
        );
        let p2 = Paragraph::new(area_info)
            .block(Block::default().title("2. Render Context").borders(Borders::ALL));
        frame.render_widget(p2, chunks[1]);

        // 3. Async Progress
        let gauge = Gauge::default()
            .block(Block::default().title("3. Async Task").borders(Borders::ALL))
            .gauge_style(Style::default().fg(if local.is_working { Color::Yellow } else { Color::Green }))
            .percent(local.progress);
        frame.render_widget(gauge, chunks[2]);

        // 4. Logs
        let items: Vec<ListItem> = local.logs.iter()
            .rev()
            .map(|l| ListItem::new(l.as_str()))
            .collect();
        let list = List::new(items)
            .block(Block::default().title("4. Event Log").borders(Borders::ALL));
        frame.render_widget(list, chunks[3]);
        
        // Help overlay (bottom)
        if !local.layout_horizontal {
             // In vertical mode, maybe don't overlay, but let's just assume simple layout for now.
             // Actually, let's render controls title on the main block border if possible? 
             // Or just verify keys work. The 'Controls' block was removed for 4-pane layout. 
             // We can put controls info in title of the whole app or just rely on user knowing.
        }
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) if key.code == KeyCode::Char('l') => {
                self.local.update(|s| s.layout_horizontal = !s.layout_horizontal);
                self.log("Layout Toggled".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('w') => {
                 let local = self.local.clone();
                 // If not already working
                 let is_working = local.read(|s| s.is_working);
                 if !is_working {
                    self.log("Async Task Started".to_string());
                    local.update(|s| { s.is_working = true; s.progress = 0; });
                    let app = cx.app.clone();
                    
                    cx.app.spawn(move |_| async move {
                        for i in 1..=10 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                            local.update(|s| s.progress = i * 10);
                        }
                        local.update(|s| { s.is_working = false; s.logs.push("Task Complete".to_string()); });
                    });
                 }
                 None
            }
             Event::Key(key) if key.code == KeyCode::Char('c') => {
                self.local.update(|s| s.logs.clear());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('j') => {
                self.state.update(|s| s.counter += 1);
                self.log("Counter ++".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('k') => {
                self.state.update(|s| s.counter -= 1);
                self.log("Counter --".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('m') => {
                Some(Action::Navigate("menu".to_string()))
            }
            Event::Key(key) if key.code == KeyCode::Esc => {
                Some(Action::Back)
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}

// A simple root component that switches between menu and pages
struct Root {
    current: Route,
    history: Vec<Route>,
    menu: Menu,
    page_a: CounterPage,
    page_b: CounterPage,
}

impl Root {
    fn new(shared_state: Entity<AppState>, cx: &AppContext) -> Self {
        Self {
            current: "menu".to_string(),
            history: Vec::new(),
            menu: Menu::new(),
            page_a: CounterPage::new("Page A", shared_state.clone(), cx.new_entity(LocalState::default())),
            page_b: CounterPage::new("Page B", shared_state, cx.new_entity(LocalState::default())),
        }
    }

    fn navigate(&mut self, route: Route) {
        if self.current != route {
            self.history.push(self.current.clone());
            self.current = route;
        }
    }

    fn go_back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
            true
        } else {
            false
        }
    }
}

impl Component for Root {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        let current = self.current.clone();
        match current.as_str() {
            "page_a" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_a.render(frame, &mut cx);
            }
            "page_b" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_b.render(frame, &mut cx);
            }
            _ => {
                let mut cx = cx.cast::<Menu>();
                self.menu.render(frame, &mut cx);
            }
        }
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        let current = self.current.clone();
        let action = match current.as_str() {
            "page_a" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_a.handle_event(event, &mut cx)
            }
            "page_b" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_b.handle_event(event, &mut cx)
            }
            _ => {
                let mut cx = cx.cast::<Menu>();
                self.menu.handle_event(event, &mut cx)
            }
        };

        if let Some(action) = action {
            match action {
                Action::Navigate(route) => {
                    self.navigate(route);
                    None
                }
                Action::Back => {
                    if self.go_back() {
                        None
                    } else {
                        None
                    }
                }
                Action::Quit => Some(Action::Quit),
                Action::Noop => None,
            }
        } else {
            None
        }
    }
}

fn main() -> std::io::Result<()> {
    let app = Application::new();

    app.run(move |cx| {
        let shared_state = cx.new_entity(AppState::default());
        let root = Root::new(shared_state, cx);
        let root = std::sync::Arc::new(std::sync::Mutex::new(root));
        cx.set_root(root);

        cx.spawn(|_| async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        });

        Ok(())
    })
}
