use ratatui::prelude::*;

pub mod div;
pub mod text;
pub mod widget;
pub mod canvas;

pub use div::{div, Div};
pub use text::{text, Text};
pub use widget::{widget, WidgetElement};
pub use canvas::{canvas, Canvas};

/// The core trait for any UI element.
pub trait Element: Send + Sync {
    /// return the width constraint for this element
    fn width(&self) -> Constraint {
        Constraint::Min(0)
    }

    /// return the height constraint for this element
    fn height(&self) -> Constraint {
        Constraint::Min(0)
    }

    /// Render the element into the given area.
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

/// Helper trait to convert types into Elements.
pub trait IntoElement {
    type Element: Element;
    fn into_element(self) -> Self::Element;
}

impl<T: Element> IntoElement for T {
    type Element = T;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Box<dyn Element> {
    fn width(&self) -> Constraint {
        self.as_ref().width()
    }
    fn height(&self) -> Constraint {
        self.as_ref().height()
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.as_mut().render(frame, area)
    }
}
