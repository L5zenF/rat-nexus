//! High‑level Application abstraction inspired by GPUI.

use crate::component::traits::{Event, Action, Component, AnyComponent};
use crate::state::{Entity, WeakEntity, EntityId};
use ratatui::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct AppContext {
    /// The root component to render, if set by the user.
    root: Arc<Mutex<Option<Entity<dyn AnyComponent>>>>,
    /// Internal: Channel to trigger a re-render.
    re_render_tx: mpsc::UnboundedSender<()>,
    /// Internal: Total frames rendered.
    frame_count: Arc<std::sync::atomic::AtomicU64>,
}

impl Clone for AppContext {
    fn clone(&self) -> Self {
        Self {
            root: Arc::clone(&self.root),
            re_render_tx: mpsc::UnboundedSender::clone(&self.re_render_tx),
            frame_count: Arc::clone(&self.frame_count),
        }
    }
}

impl AppContext {
    /// Create a new entity with the given value.
    pub fn new_entity<T>(&self, value: T) -> Entity<T>
    where
        T: Send + Sync + 'static,
    {
        Entity::new(value)
    }

    /// Schedule a task to be executed later.
    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let cx = AppContext::clone(self);
        tokio::spawn(async move {
            f(cx).await;
        });
    }

    /// Spawn a task and return a handle that can be used to cancel it.
    pub fn spawn_task<F, Fut>(&self, f: F) -> crate::task::TaskHandle
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let cx = AppContext::clone(self);
        let join_handle = tokio::spawn(async move {
            f(cx).await;
        });
        crate::task::TaskHandle::new(join_handle.abort_handle())
    }

    /// Set the root component of the application.
    pub fn set_root(&self, root: Entity<dyn AnyComponent>) -> crate::Result<()> {
        let mut guard = self.root.lock().map_err(|_| crate::Error::LockPoisoned)?;
        *guard = Some(root);
        self.refresh();
        Ok(())
    }

    /// Trigger a re-render.
    pub fn refresh(&self) {
        let _ = self.re_render_tx.send(());
    }

    /// Get the total number of frames rendered.
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// A specialized context passed to component methods.
/// Note: For rendering area, use `frame.area()` instead.
pub struct Context<V: ?Sized + Send + Sync> {
    pub app: AppContext,
    pub handle: Option<WeakEntity<V>>,
}

impl<V: ?Sized + Send + Sync> Context<V> {
    pub fn new(app: AppContext) -> Self {
        Self {
            app,
            handle: None,
        }
    }

    pub fn with_handle(app: AppContext, handle: WeakEntity<V>) -> Self {
        Self {
            app,
            handle: Some(handle),
        }
    }

    /// Access the underlying AppContext.
    pub fn app(&self) -> &AppContext {
        &self.app
    }

    /// Subscribe to an entity's changes.
    pub fn subscribe<T>(&mut self, entity: &Entity<T>)
    where T: Send + Sync + 'static
    {
        let mut rx = entity.subscribe();
        let tx = mpsc::UnboundedSender::clone(&self.app.re_render_tx);
        tokio::spawn(async move {
            while rx.changed().await.is_ok() {
                let _ = tx.send(());
            }
        });
    }

    /// Watch an entity: subscribe to changes and read the current value.
    /// This is a convenience method that combines `subscribe` and `entity.read`.
    ///
    /// # Example
    /// ```ignore
    /// fn render(&mut self, frame: &mut Frame, cx: &mut Context<Self>) {
    ///     // Instead of:
    ///     // cx.subscribe(&self.state);
    ///     // let counter = self.state.read(|s| s.counter).unwrap();
    ///
    ///     // Use:
    ///     let counter = cx.watch(&self.state, |s| s.counter).unwrap();
    /// }
    /// ```
    pub fn watch<T, F, R>(&mut self, entity: &Entity<T>, f: F) -> Option<R>
    where
        T: Send + Sync + 'static,
        F: FnOnce(&T) -> R,
    {
        self.subscribe(entity);
        entity.read(f).ok()
    }

    /// Spawn a task using the application context.
    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.app.spawn(f);
    }

    /// Spawn a task and return a handle that can be used to cancel it.
    /// Use this with `TaskTracker` for proper lifecycle management.
    pub fn spawn_task<F, Fut>(&self, f: F) -> crate::task::TaskHandle
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.app.spawn_task(f)
    }

    /// Cast this context to another view type.
    pub fn cast<U: ?Sized + Send + Sync + 'static>(&self) -> Context<U> {
        Context {
            app: AppContext::clone(&self.app),
            handle: None,
        }
    }

    /// Get the entity ID of the component this context is bound to.
    /// Returns None if the context was not created with a handle.
    pub fn entity_id(&self) -> Option<EntityId> {
        self.handle.as_ref().map(|h| h.entity_id())
    }

    /// Get a weak handle to the component this context is bound to.
    /// Returns None if the context was not created with a handle.
    pub fn weak_entity(&self) -> Option<WeakEntity<V>> {
        self.handle.clone()
    }

    /// Get a strong handle to the component this context is bound to.
    /// Returns None if the context was not created with a handle or if the entity was dropped.
    pub fn entity(&self) -> Option<Entity<V>> {
        self.handle.as_ref().and_then(|h| h.upgrade())
    }

    /// Explicitly trigger a re-render.
    pub fn notify(&self) {
        self.app.refresh();
    }
}

/// EventContext for event handling, currently identical to Context but renamed for clarity.
pub type EventContext<V> = Context<V>;

/// Main application handle.
pub struct Application;

impl Application {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self
    }

    /// Run the application with the given closure that receives a context.
    pub fn run<F>(self, setup: F) -> anyhow::Result<()>
    where
        F: FnOnce(&AppContext) -> anyhow::Result<()>,
    {
        let rt = Runtime::new().map_err(|e| anyhow::anyhow!("Failed to start tokio: {}", e))?;
        let (re_render_tx, re_render_rx) = mpsc::unbounded_channel();
        let root = Arc::new(Mutex::new(None));
        let app_context = AppContext {
            root: Arc::clone(&root),
            re_render_tx,
            frame_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        };

        let _guard = rt.enter();
        setup(&app_context)?;
        drop(_guard);

        let actual_root: Entity<dyn AnyComponent> = {
            let guard = root.lock().map_err(|_| anyhow::anyhow!("Root mutex poisoned"))?;
            guard.as_ref().map(Entity::clone).unwrap_or_else(|| {
                Entity::from_arc(Arc::new(Mutex::new(DummyView)) as Arc<Mutex<dyn AnyComponent>>)
            })
        };

        let result = rt.block_on(async move {
            self.run_loop(app_context, actual_root, re_render_rx).await
        });

        // Ensure we don't hang forever on background tasks (like infinite loops in components)
        rt.shutdown_timeout(Duration::from_millis(100));

        result
    }

    async fn run_loop(&self, app: AppContext, root: Entity<dyn AnyComponent>, re_render_rx: mpsc::UnboundedReceiver<()>) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, event::EnableFocusChange)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Lifecycle: Call on_mount (first time) and on_enter (entering view) on the root component
        {
            let weak = root.downgrade();
            let mut cx = Context::<dyn AnyComponent>::with_handle(AppContext::clone(&app), weak);
            root.update(|comp| {
                comp.on_mount_any(&mut cx);
                comp.on_enter_any(&mut cx);
            }).map_err(|_| anyhow::anyhow!("Root mutex poisoned during on_mount"))?;
        }

        let result = self.run_app_loop(app, &mut terminal, root, re_render_rx).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            event::DisableFocusChange
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app_loop(
        &self,
        app: AppContext,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        root: Entity<dyn AnyComponent>,
        mut re_render_rx: mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        // Initial render
        let _ = app.re_render_tx.send(());

        // Dedicated event polling task to avoid blocking the main loop
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        tokio::task::spawn_blocking(move || {
            loop {
                // Check if the main loop is still interested in events
                if event_tx.is_closed() {
                    break;
                }

                // Poll at ~60fps (16.67ms) for smooth animations
                match event::poll(Duration::from_millis(16)) {
                    Ok(true) => {
                        if let Ok(e) = event::read() {
                            if event_tx.send(e).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
        });

        loop {
            tokio::select! {
                // Prioritize event handling for lower latency
                biased;

                Some(crossterm_event) = event_rx.recv() => {
                    let internal_event = match crossterm_event {
                        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Some(Event::Key(key)),
                        CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                        CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
                        CrosstermEvent::FocusGained => Some(Event::FocusGained),
                        CrosstermEvent::FocusLost => Some(Event::FocusLost),
                        CrosstermEvent::Paste(s) => Some(Event::Paste(s)),
                        _ => None,
                    };

                    if let Some(event) = internal_event {
                        let weak = root.downgrade();
                        let mut cx = EventContext::<dyn AnyComponent>::with_handle(AppContext::clone(&app), weak);

                        let action = root.update(|comp| {
                            comp.handle_event_any(event, &mut cx)
                        }).map_err(|_| anyhow::anyhow!("Root mutex poisoned during event"))?;

                        app.refresh(); // Trigger refresh after any event handling

                        if let Some(action) = action {
                            match action {
                                Action::Quit => {
                                    let weak = root.downgrade();
                                    let mut cx = Context::<dyn AnyComponent>::with_handle(AppContext::clone(&app), weak);
                                    root.update(|comp| comp.on_shutdown_any(&mut cx))
                                        .map_err(|_| anyhow::anyhow!("Root mutex poisoned during shutdown"))?;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                _ = re_render_rx.recv() => {
                    // Drain all pending refresh requests to compact them into a single frame
                    while re_render_rx.try_recv().is_ok() {}

                    let weak = root.downgrade();
                    terminal.draw(|frame| {
                        app.frame_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let mut cx = Context::<dyn AnyComponent>::with_handle(AppContext::clone(&app), weak);
                        root.update(|comp| comp.render_any(frame, &mut cx))
                            .expect("Root mutex poisoned during render");
                    })?;
                }
            }
        }
    }
}

struct DummyView;

impl Component for DummyView {
    fn render(&mut self, frame: &mut ratatui::Frame, _cx: &mut Context<Self>) {
        let paragraph = ratatui::widgets::Paragraph::new("No component set")
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, frame.area());
    }
}
