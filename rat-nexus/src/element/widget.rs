use ratatui::prelude::*;
use ratatui::widgets::Widget;
use crate::element::Element;

#[derive(Clone)]
pub struct WidgetElement<W> {
    widget: W,
    width_constraint: Constraint,
    height_constraint: Constraint,
}

impl<W> WidgetElement<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            width_constraint: Constraint::Min(0),
            height_constraint: Constraint::Min(0), // Default to flex/fill
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
    
    pub fn w_percent(mut self, p: u16) -> Self {
        self.width_constraint = Constraint::Percentage(p);
        self
    }

    pub fn h_percent(mut self, p: u16) -> Self {
        self.height_constraint = Constraint::Percentage(p);
        self
    }
}

impl<W: Widget + Clone + Send + Sync + 'static> Element for WidgetElement<W> {
    fn width(&self) -> Constraint {
        self.width_constraint
    }

    fn height(&self) -> Constraint {
        self.height_constraint
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.widget.clone(), area);
    }
}

pub fn widget<W: Widget + Clone + Send + Sync + 'static>(w: W) -> WidgetElement<W> {
    WidgetElement::new(w)
}
