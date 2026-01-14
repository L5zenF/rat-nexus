use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};
use crate::element::{Element, IntoElement};

pub struct Text {
    pub content: String,
    pub style: Style,
    pub style_fn: Option<Box<dyn Fn(Style) -> Style + Send + Sync>>,
    pub alignment: Alignment,
    pub wrap: bool,
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
        }
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
        self
    }

    pub fn align_right(mut self) -> Self {
        self.alignment = Alignment::Right;
        self
    }
}

impl Element for Text {
    fn width(&self) -> Constraint {
        Constraint::Length(self.content.len() as u16)
    }

    fn height(&self) -> Constraint {
        Constraint::Length(1)
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
