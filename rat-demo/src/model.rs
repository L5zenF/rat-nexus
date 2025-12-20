//! Application state models demonstrating Entity reactive state management.

/// Global application state shared across all pages.
/// Changes to this state trigger re-renders in all subscribed components.
#[derive(Clone)]
pub struct AppState {
    pub counter: i32,
    pub history: Vec<i64>,  // Changed to i64 to support negative values
    pub min_value: i32,
    pub max_value: i32,
    pub step: i32,
    pub theme: Theme,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            history: vec![0; 50],
            min_value: 0,
            max_value: 0,
            step: 1,
            theme: Theme::default(),
        }
    }
}

/// Theme configuration for the application.
#[derive(Clone, Copy, Default, PartialEq)]
pub enum Theme {
    #[default]
    Cyan,
    Green,
    Magenta,
    Yellow,
}

impl Theme {
    pub fn next(&self) -> Self {
        match self {
            Theme::Cyan => Theme::Green,
            Theme::Green => Theme::Magenta,
            Theme::Magenta => Theme::Yellow,
            Theme::Yellow => Theme::Cyan,
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            Theme::Cyan => ratatui::style::Color::Cyan,
            Theme::Green => ratatui::style::Color::Green,
            Theme::Magenta => ratatui::style::Color::Magenta,
            Theme::Yellow => ratatui::style::Color::Yellow,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theme::Cyan => "Cyan",
            Theme::Green => "Green",
            Theme::Magenta => "Magenta",
            Theme::Yellow => "Yellow",
        }
    }
}

/// Local state for Counter pages.
/// Each Counter page has its own independent local state.
#[derive(Clone)]
pub struct LocalState {
    pub layout_horizontal: bool,
    pub logs: Vec<String>,
    pub progress: u16,
    pub is_working: bool,
    pub current_time: String,
    pub system_load: Vec<u64>,
    pub pulse_inc: u8,
    pub pulse_dec: u8,
    pub fps: f64,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            layout_horizontal: false,
            logs: vec!["System initialized".to_string()],
            progress: 0,
            is_working: false,
            current_time: "--:--:--".to_string(),
            system_load: vec![0; 20],
            pulse_inc: 0,
            pulse_dec: 0,
            fps: 0.0,
        }
    }
}

/// State for the System Monitor page.
#[derive(Clone)]
pub struct MonitorState {
    pub cpu_history: Vec<u64>,
    pub memory_history: Vec<u64>,
    pub network_in: Vec<u64>,
    pub network_out: Vec<u64>,
    pub disk_usage: u16,
    pub cpu_cores: Vec<u16>,
    pub processes: Vec<ProcessInfo>,
    pub uptime_secs: u64,
}

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory: f32,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            cpu_history: vec![0; 60],
            memory_history: vec![0; 60],
            network_in: vec![0; 30],
            network_out: vec![0; 30],
            disk_usage: 45,
            cpu_cores: vec![0; 8],
            processes: vec![
                ProcessInfo { pid: 1, name: "init".into(), cpu: 0.1, memory: 0.5 },
                ProcessInfo { pid: 100, name: "rat-demo".into(), cpu: 2.5, memory: 1.2 },
                ProcessInfo { pid: 200, name: "tokio-rt".into(), cpu: 1.8, memory: 0.8 },
                ProcessInfo { pid: 300, name: "crossterm".into(), cpu: 0.5, memory: 0.3 },
            ],
            uptime_secs: 0,
        }
    }
}

/// State for the Widget Showcase page.
#[derive(Clone)]
pub struct ShowcaseState {
    pub selected_tab: usize,
    pub table_selected: usize,
    pub scroll_offset: u16,
    pub list_items: Vec<String>,
    pub chart_data: Vec<(f64, f64)>,
    pub bar_data: Vec<(&'static str, u64)>,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            selected_tab: 0,
            table_selected: 0,
            scroll_offset: 0,
            list_items: vec![
                "Entity<T> - Reactive state container".into(),
                "Component - Lifecycle-aware UI building block".into(),
                "Context<V> - GPUI-style component binding".into(),
                "TaskTracker - Cancellable async task management".into(),
                "Router - Type-safe navigation".into(),
                "EntityId - Unique component identification".into(),
                "WeakEntity - Safe async references".into(),
                "AppContext - Global application context".into(),
            ],
            chart_data: (0..50).map(|i| {
                let x = i as f64;
                let y = (x * 0.2).sin() * 50.0 + 50.0;
                (x, y)
            }).collect(),
            bar_data: vec![
                ("Mon", 65),
                ("Tue", 80),
                ("Wed", 45),
                ("Thu", 90),
                ("Fri", 70),
                ("Sat", 30),
                ("Sun", 55),
            ],
        }
    }
}
