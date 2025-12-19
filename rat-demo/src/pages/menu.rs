use rat_nexus::{Component, Context, EventContext, Event, Action, Route};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::style::{Style, Modifier};
use crossterm::event::KeyCode;

pub struct Menu {
    selected: usize,
    options: Vec<(&'static str, Route)>,
}

impl Menu {
    pub fn new() -> Self {
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
