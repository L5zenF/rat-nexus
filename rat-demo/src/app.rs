use rat_nexus::prelude::*;
use rat_nexus::define_app;
use crate::pages::{Menu, MonitorPage, TimerPage, ParticlesPage, FlappyPage, TicTacToePage, LogPage};

// Define Root with all pages - fully auto-generated routing & lifecycle!
// Supports both simple syntax (below) and full syntax with #[Root(default=Menu)]
define_app! {
    Menu => menu: Menu,
    Monitor => monitor: MonitorPage,
    Timer => timer: TimerPage,
    Particles => particles: ParticlesPage,
    Flappy => flappy: FlappyPage,
    Tictactoe => tictactoe: TicTacToePage,
    Logs => logs: LogPage,
}
