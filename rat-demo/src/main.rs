//! Example TUI application using the rat-setup framework.

mod model;
mod pages;
mod app;

use rat_nexus::{Application, Entity, AnyComponent};
use crate::model::AppState;
use crate::app::Root;

fn main() -> anyhow::Result<()> {
    let app = Application::new();

    app.run(move |cx| {
        // Store shared state in context - pages will access via cx.get()
        let shared_state = cx.new_entity(AppState::default());
        cx.set(shared_state);

        // Create root app - the macro generates new() that calls Page::build() for each page
        let root = Root::new(cx);

        // Wrap the root component in an Entity for GPUI-style state management
        let root: Entity<dyn AnyComponent> = Entity::from_arc(
            std::sync::Arc::new(std::sync::Mutex::new(root)) as std::sync::Arc<std::sync::Mutex<dyn AnyComponent>>
        );
        cx.set_root(root)?;

        Ok(())
    })
}
