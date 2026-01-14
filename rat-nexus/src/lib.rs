pub mod application;
pub mod component;
pub mod state;
pub mod router;
pub mod task;
pub mod error;

pub mod element;

pub use error::{Error, Result};

// Re-export common types for convenience
pub use application::{Application, AppContext, Context, EventContext};
pub use component::{Component, traits::{Event, Action, AnyComponent}};
pub use state::{Entity, WeakEntity, EntityId};
pub use router::{Route, Router};
pub use task::{TaskHandle, TaskTracker};
pub use element::{Element, IntoElement, div, text, Div, Text};

// Re-export paste for macro usage
pub use paste;

pub mod prelude {
    pub use crate::application::{Application, AppContext, Context, EventContext};
    pub use crate::component::{Component, traits::{Event, Action, AnyComponent}};
    pub use crate::state::{Entity, WeakEntity, EntityId};
    pub use crate::router::{Route, Router};
    pub use crate::element::{Element, IntoElement, div, text, Div, Text, widget, WidgetElement, canvas, Canvas};
    pub use crate::task::{TaskHandle, TaskTracker};
    // Re-export commonly used ratatui types for convenience
    pub use ratatui::prelude::{Constraint, Direction, Rect, Frame, Color, Modifier, Style};
}
