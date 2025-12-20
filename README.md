# Rat-Nexus

A modern reactive TUI framework for Rust, inspired by [GPUI](https://github.com/zed-industries/zed), built on [Ratatui](https://github.com/ratatui-org/ratatui).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

![demo](./asserts/bkg.png)

## Features

- **Reactive State Management** — `Entity<T>` provides observable state with automatic UI updates
- **GPUI-Style Context** — Components access themselves via `Context` with `entity_id()`, `entity()`, `weak_entity()`
- **Clear Lifecycle Hooks** — `on_mount`, `on_enter`, `on_exit`, `on_shutdown` for precise control
- **Cancellable Async Tasks** — `TaskTracker` prevents task leaks when components are destroyed
- **Type-Safe Routing** — Compile-time checked routes with `define_routes!` macro

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rat-nexus = { path = "rat-nexus" }
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## Quick Start

A minimal counter application:

```rust
use crossterm::event::KeyCode;
use rat_nexus::{Action, AnyComponent, Application, Component, Context, Entity, Event, EventContext};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, Paragraph},
};
use std::sync::{Arc, Mutex};

// 1. Define state
struct CounterState {
    count: i32,
}

// 2. Define component
struct Counter {
    state: Entity<CounterState>,
}

// 3. Implement Component trait
impl Component for Counter {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // watch = subscribe + read, auto re-renders on state change
        let count = cx.watch(&self.state, |s| s.count).unwrap_or(0);

        let area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(30),
            Constraint::Fill(1),
        ])
        .split(frame.area())[1];

        frame.render_widget(
            Paragraph::new(format!("Count: {count}"))
                .alignment(Alignment::Center)
                .block(Block::bordered().title(" Counter ").border_type(BorderType::Rounded)),
            area,
        );
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') => { let _ = self.state.update(|s| s.count += 1); }
                KeyCode::Char('k') => { let _ = self.state.update(|s| s.count -= 1); }
                KeyCode::Char('q') => return Some(Action::Quit),
                _ => {}
            }
        }
        None
    }
}

// 4. Run application
fn main() -> anyhow::Result<()> {
    Application::new().run(|cx| {
        let state = cx.new_entity(CounterState { count: 0 });
        let counter = Counter { state };

        // Wrap component in Entity<dyn AnyComponent>
        let root: Entity<dyn AnyComponent> = Entity::from_arc(
            Arc::new(Mutex::new(counter)) as Arc<Mutex<dyn AnyComponent>>
        );
        cx.set_root(root)?;
        Ok(())
    })
}
```

## Core Concepts

### Entity: Reactive State

`Entity<T>` is a reactive state container that notifies subscribers on changes:

```rust
// Create
let state = cx.new_entity(MyState::default());

// Each entity has a unique ID
let id: EntityId = state.entity_id();

// Update (automatically triggers re-render)
state.update(|s| s.counter += 1)?;

// Read
let value = state.read(|s| s.counter)?;

// Subscribe + Read (recommended in render)
let value = cx.watch(&state, |s| s.counter).unwrap();

// Create weak reference
let weak: WeakEntity<MyState> = state.downgrade();
```

### Context: GPUI-Style Component Access

`Context<V>` binds to a component and provides access to its entity:

```rust
impl Component for MyComponent {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // Get component's EntityId
        if let Some(id) = cx.entity_id() {
            println!("Component mounted with ID: {}", id);
        }

        // Get weak reference for async tasks
        if let Some(weak) = cx.weak_entity() {
            cx.spawn(move |app| async move {
                // Safe access to component from async context
                if let Some(entity) = weak.upgrade() {
                    entity.update(|comp| {
                        // Update component state
                    });
                }
                app.refresh();
            });
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // Access app context
        let frame_count = cx.app.frame_count();

        // Subscribe to state changes
        let value = cx.watch(&self.state, |s| s.value).unwrap();
    }
}
```

### Component Lifecycle

```rust
impl Component for MyPage {
    /// Called once when first mounted
    /// Use for: starting background tasks, initializing resources
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        let handle = cx.spawn_task(|app| async move {
            loop {
                app.refresh();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        self.tasks.track(handle);
    }

    /// Called each time the view becomes active
    /// Use for: resetting transient state, refreshing data
    fn on_enter(&mut self, _cx: &mut Context<Self>) {
        // ...
    }

    /// Called when leaving the view
    /// Use for: pausing tasks, saving state
    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    /// Called before application exits
    /// Use for: persistence, cleanup
    fn on_shutdown(&mut self, _cx: &mut Context<Self>) {
        // ...
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // Use frame.area() to get rendering area
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        None
    }
}
```

### TaskTracker: Cancellable Async Tasks

Prevents task leaks with automatic cleanup on component destruction:

```rust
use rat_nexus::TaskTracker;

struct MyComponent {
    tasks: TaskTracker,
}

impl Component for MyComponent {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // spawn_task returns a cancellable TaskHandle
        let handle = cx.spawn_task(|app| async move {
            loop {
                app.refresh();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        // Track the task
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        // Cancel all tracked tasks
        self.tasks.abort_all();
    }
}

// TaskTracker implements Drop, auto-aborts on destruction
```

### Router: Type-Safe Routing

Compile-time route checking eliminates typos:

```rust
use rat_nexus::{define_routes, Router};

// Define route enum
define_routes! {
    Menu,
    Counter,
    Settings,
    Snake,
}

// Usage
let mut router = Router::new(Route::Menu);

router.navigate(Route::Counter);  // Compile-time type checking

if router.can_go_back() {
    router.go_back();
}

match router.current() {
    Route::Menu => { /* ... */ }
    Route::Counter => { /* ... */ }
    _ => {}
}
```

## API Reference

### Context Methods

| Method | Description |
|--------|-------------|
| `cx.entity_id()` | Get EntityId of bound component |
| `cx.entity()` | Get strong Entity handle |
| `cx.weak_entity()` | Get weak Entity handle |
| `cx.watch(&entity, \|s\| ...)` | Subscribe and read state |
| `cx.subscribe(&entity)` | Subscribe to state changes only |
| `cx.spawn(f)` | Spawn background task (non-cancellable) |
| `cx.spawn_task(f)` | Spawn background task (returns TaskHandle) |
| `cx.notify()` | Manually trigger re-render |
| `cx.cast::<U>()` | Convert Context type for child components |
| `cx.app` | Access AppContext |

### Component Lifecycle

| Method | When Called | Use Case |
|--------|-------------|----------|
| `on_mount` | First mount (once) | Start background tasks |
| `on_enter` | Each view entry | Refresh data |
| `on_exit` | Leaving view | Pause/cancel tasks |
| `on_shutdown` | App exit | Persistence/cleanup |
| `render` | Each re-render | Draw UI |
| `handle_event` | Event received | Handle input |

### Entity Methods

| Method | Description |
|--------|-------------|
| `Entity::new(value)` | Create new entity |
| `Entity::from_arc(arc)` | Create from Arc<Mutex<T>> |
| `entity.entity_id()` | Get unique EntityId |
| `entity.update(\|s\| ...)` | Update state, notify subscribers |
| `entity.read(\|s\| ...)` | Read state |
| `entity.downgrade()` | Get WeakEntity |
| `entity.subscribe()` | Get change receiver |

### Public Exports

```rust
pub use rat_nexus::{
    // Application
    Application, AppContext, Context, EventContext,
    // Component
    Component, Event, Action, AnyComponent,
    // State
    Entity, WeakEntity, EntityId,
    // Router
    Router, Route, define_routes,
    // Tasks
    TaskHandle, TaskTracker,
    // Error
    Error, Result,
};
```

## Running the Demo

```bash
# Run demo application
cargo run

# Controls
# ↑/↓/Enter - Navigate menu
# j/k       - Increment/decrement counter
# w         - Start async worker
# m         - Return to menu
# q         - Quit

# Snake game
# ←↑↓→/wasd - Move
# Space     - Pause
# r         - Restart
```

## Project Structure

```
.
├── rat-nexus/              # Core framework
│   └── src/
│       ├── application.rs  # Application, Context, AppContext
│       ├── component/      # Component trait, AnyComponent
│       ├── state/          # Entity, WeakEntity, EntityId
│       ├── router/         # Router, define_routes!
│       ├── task.rs         # TaskHandle, TaskTracker
│       ├── error.rs        # Error types
│       └── lib.rs          # Public API
│
└── rat-demo/               # Example application
    └── src/
        ├── pages/          # Page components
        ├── model.rs        # State definitions
        ├── app.rs          # Root component
        └── main.rs         # Entry point
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Application                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │  AppContext │  │   Entity    │  │   TaskTracker   │  │
│  │  - refresh  │  │  - EntityId │  │  - track()      │  │
│  │  - spawn    │  │  - update() │  │  - abort_all()  │  │
│  │  - set_root │  │  - read()   │  │                 │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │                Context<V>                        │    │
│  │  - entity_id() / entity() / weak_entity()       │    │
│  │  - watch() / subscribe() / notify()             │    │
│  │  - spawn() / spawn_task()                       │    │
│  └─────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │              Component Trait                     │    │
│  │  on_mount → on_enter → render ⟷ handle_event   │    │
│  │                         ↓                        │    │
│  │                      on_exit → on_shutdown       │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## License

MIT
