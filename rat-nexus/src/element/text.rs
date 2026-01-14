use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};
use crate::element::{Element, IntoElement};

pub struct Text {
    pub content: String,
    pub style: Style,
    pub style_fn: Option<Box<dyn Fn(Style) -> Style + Send + Sync>>,
    pub alignment: Alignment,
    pub wrap: bool,
    pub width_constraint: Constraint,
    pub height_constraint: Constraint,
}

pub fn text(content: impl Into<String>) -> Text {
    Text::new(content)
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: Style::default(),
            style_fn: None,
            alignment: Alignment::Left,
            wrap: false,
            width_constraint: Constraint::Min(0), // Default to flex
            height_constraint: Constraint::Length(1),
        }
    }

    pub fn w_full(mut self) -> Self {
        self.width_constraint = Constraint::Percentage(100);
        self
    }

    pub fn h_full(mut self) -> Self {
        self.height_constraint = Constraint::Percentage(100);
        self
    }

    pub fn w(mut self, length: u16) -> Self {
        self.width_constraint = Constraint::Length(length);
        self
    }

    pub fn h(mut self, length: u16) -> Self {
        self.height_constraint = Constraint::Length(length);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.style = self.style.add_modifier(Modifier::BOLD);
        self
    }
    
    pub fn align_center(mut self) -> Self {
        self.alignment = Alignment::Center;
        self.width_constraint = Constraint::Min(0); // Ensure it takes space to align
        self
    }

    pub fn align_right(mut self) -> Self {
        self.alignment = Alignment::Right;
        self.width_constraint = Constraint::Min(0);
        self
    }
}

impl Element for Text {
    fn width(&self) -> Constraint {
        self.width_constraint
    }

    fn height(&self) -> Constraint {
        self.height_constraint
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let mut p = Paragraph::new(self.content.clone())
            .style(self.style)
            .alignment(self.alignment);
        
        if self.wrap {
             p = p.wrap(Wrap { trim: true });
        }

        frame.render_widget(p, area);
    }
}

impl IntoElement for String {
    type Element = Text;
    fn into_element(self) -> Self::Element {
        Text::new(self)
    }
}

impl IntoElement for &str {
    type Element = Text;
    fn into_element(self) -> Self::Element {
        Text::new(self.to_string())
    }
}
