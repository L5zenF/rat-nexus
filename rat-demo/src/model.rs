#[derive(Default, Clone)]
pub struct AppState {
    pub counter: i32,
}

#[derive(Clone)]
pub struct LocalState {
    pub layout_horizontal: bool,
    pub logs: Vec<String>,
    pub progress: u16,
    pub is_working: bool,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            layout_horizontal: false,
            logs: vec!["Initialized".to_string()],
            progress: 0,
            is_working: false,
        }
    }
}
