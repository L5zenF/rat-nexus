use ratatui::prelude::*;
use crate::element::Element;

pub struct Canvas {
    painter: Option<Box<dyn FnOnce(&mut Frame, Rect) + Send + Sync>>,
    width_constraint: Constraint,
    height_constraint: Constraint,
}

impl Canvas {
    pub fn new(painter: impl FnOnce(&mut Frame, Rect) + Send + Sync + 'static) -> Self {
        Self {
            painter: Some(Box::new(painter)),
            width_constraint: Constraint::Percentage(100),
            height_constraint: Constraint::Percentage(100),
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
}

impl Element for Canvas {
    fn width(&self) -> Constraint {
        self.width_constraint
    }

    fn height(&self) -> Constraint {
        self.height_constraint
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(painter) = self.painter.take() {
            painter(frame, area);
        }
    }
}

pub fn canvas(painter: impl FnOnce(&mut Frame, Rect) + Send + Sync + 'static) -> Canvas {
    Canvas::new(painter)
}
