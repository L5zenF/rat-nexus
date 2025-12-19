pub mod application;
pub mod component;
pub mod state;
pub mod router;

// Re-export common types for convenience
pub use application::{Application, AppContext, Context, EventContext};
pub use component::{Component, traits::{Event, Action, AnyComponent}};
pub use state::{Entity, WeakEntity};
pub use router::traits::{Route, Router};