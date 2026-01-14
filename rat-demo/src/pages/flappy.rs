//! Flappy Bird - Classic arcade game clone
//! Showcases: Real-time game loop, collision detection, Entity state, Componentization

use rat_nexus::prelude::*;
use ratatui::{
    widgets::{Block, Borders, BorderType, canvas::{Canvas as RatatuiCanvas, Rectangle, Points, Circle, Line as CanvasLine, Context as CanvasContext}},
    style::{Style, Color},
    text::Line,
};
use crossterm::event::KeyCode;

const GRAVITY: f64 = 0.22;
const JUMP_FORCE: f64 = 1.6;
const PIPE_GAP: f64 = 15.0;
const PIPE_WIDTH: f64 = 5.0;
const PIPE_SPEED: f64 = 0.8;

// ============================================
// Bird Component - Drawn with particles
// ============================================
#[derive(Clone)]
pub struct Bird {
    pub x: f64,
    pub y: f64,
    pub vy: f64,
    pub radius: f64,
    pub alive: bool,
}

impl Bird {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, vy: 0.0, radius: 1.2, alive: true }
    }

    pub fn update(&mut self) {
        self.vy -= GRAVITY;
        self.y += self.vy;
    }

    pub fn flap(&mut self) {
        self.vy = JUMP_FORCE;
    }

    pub fn check_bounds(&mut self, ground: f64, ceiling: f64) {
        if self.y < ground + self.radius {
            self.y = ground + self.radius;
            self.alive = false;
        }
        if self.y > ceiling - self.radius {
            self.y = ceiling - self.radius;
            self.vy = 0.0;
        }
    }

    pub fn collides_with_pipe(&self, pipe_x: f64, gap_y: f64) -> bool {
        if self.x + self.radius > pipe_x && self.x - self.radius < pipe_x + PIPE_WIDTH {
            let in_gap = self.y > gap_y - PIPE_GAP / 2.0 + self.radius
                && self.y < gap_y + PIPE_GAP / 2.0 - self.radius;
            return !in_gap;
        }
        false
    }

    pub fn reset(&mut self, y: f64) {
        self.y = y;
        self.vy = 0.0;
        self.alive = true;
    }

    /// Render bird using emoji + particles (~64 particles for effects)
    pub fn render(&self, ctx: &mut CanvasContext) {
        let x = self.x;
        let y = self.y;
        let r = self.radius;

        if !self.alive {
            // === Dead Bird - Skull emoji + grayed out bits ===
            ctx.print(x - 0.5, y, Line::styled("💀", Style::default()));
            ctx.draw(&Circle { x, y, radius: r, color: Color::DarkGray });
            return;
        }

        // === 1. Body (Golden Yellow) ===
        ctx.draw(&Circle {
            x, y,
            radius: r,
            color: Color::Rgb(255, 204, 0),
        });

        // === 2. Eye (White with Black Pupil) ===
        ctx.draw(&Points {
            coords: &[(x + r * 0.4, y + r * 0.3)],
            color: Color::White,
        });
        ctx.draw(&Points {
            coords: &[(x + r * 0.5, y + r * 0.35)],
            color: Color::Black,
        });

        // === 3. Beak (Sharp Orange) ===
        // Top Part
        ctx.draw(&CanvasLine {
            x1: x + r - 0.1, y1: y + 0.1,
            x2: x + r + 0.6, y2: y,
            color: Color::Rgb(255, 102, 0),
        });
        // Bottom Part
        ctx.draw(&CanvasLine {
            x1: x + r - 0.1, y1: y - 0.1,
            x2: x + r + 0.5, y2: y - 0.05,
            color: Color::Rgb(255, 102, 0),
        });

        // === 4. Wing (Light Yellow, Animating) ===
        let wing_offset = (self.vy * 0.8).clamp(-r * 0.8, r * 0.8);
        ctx.draw(&Circle {
            x: x - r * 0.3,
            y: y + wing_offset,
            radius: r * 0.5,
            color: Color::Rgb(255, 255, 153),
        });
        // Wing details
        ctx.draw(&CanvasLine {
            x1: x - r * 0.6, y1: y + wing_offset,
            x2: x - r * 0.1, y2: y + wing_offset,
            color: Color::Rgb(220, 220, 0),
        });

        // === 5. Tail (Little tuft) ===
        ctx.draw(&CanvasLine {
            x1: x - r, y1: y,
            x2: x - r - 0.5, y2: y + 0.2,
            color: Color::Rgb(255, 204, 0),
        });

        // === Particle Effects (Keep the existing juice) ===
        // Wing particles (~12)
        let mut wing_particles = vec![];
        for i in 0..4 {
            let t = i as f64 / 3.0;
            wing_particles.push((x - 1.2 - t, y + wing_offset + t * 0.2));
        }
        ctx.draw(&Points { coords: &wing_particles, color: Color::Rgb(255, 240, 100) });

        // Sparkle trail when moving fast
        if self.vy.abs() > 0.4 {
            let t = (x + y).sin() * 0.5;
            ctx.draw(&Points {
                coords: &[(x - 2.5, y + t), (x - 3.2, y - t)],
                color: Color::Rgb(255, 255, 255),
            });
        }
    }
}

// ============================================
// Pipe Component
// ============================================
#[derive(Clone)]
pub struct Pipe {
    pub x: f64,
    pub gap_y: f64,
    pub passed: bool,
}

impl Pipe {
    pub fn new(x: f64, gap_y: f64) -> Self {
        Self { x, gap_y, passed: false }
    }

    pub fn update(&mut self) {
        self.x -= PIPE_SPEED;
    }

    pub fn render(&self, ctx: &mut CanvasContext) {
        // Top pipe
        ctx.draw(&Rectangle {
            x: self.x,
            y: self.gap_y + PIPE_GAP / 2.0,
            width: PIPE_WIDTH,
            height: 50.0 - (self.gap_y + PIPE_GAP / 2.0),
            color: Color::Green,
        });
        // Bottom pipe
        ctx.draw(&Rectangle {
            x: self.x,
            y: 2.0,
            width: PIPE_WIDTH,
            height: (self.gap_y - PIPE_GAP / 2.0 - 2.0).max(0.0),
            color: Color::Green,
        });
        // Pipe caps
        ctx.draw(&Rectangle {
            x: self.x - 0.5,
            y: self.gap_y + PIPE_GAP / 2.0 - 1.0,
            width: PIPE_WIDTH + 1.0,
            height: 1.2,
            color: Color::LightGreen,
        });
        ctx.draw(&Rectangle {
            x: self.x - 0.5,
            y: self.gap_y - PIPE_GAP / 2.0,
            width: PIPE_WIDTH + 1.0,
            height: 1.2,
            color: Color::LightGreen,
        });
    }
}

// ============================================
// Game State
// ============================================
#[derive(Clone)]
pub struct FlappyState {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: u32,
    high_score: u32,
    started: bool,
    tick: u64,
}

impl Default for FlappyState {
    fn default() -> Self {
        Self {
            bird: Bird::new(20.0, 25.0),
            pipes: vec![],
            score: 0,
            high_score: 0,
            started: false,
            tick: 0,
        }
    }
}

impl FlappyState {
    fn reset(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.bird.reset(25.0);
        self.pipes.clear();
        self.score = 0;
        self.started = false;
        self.tick = 0;
    }
}

#[derive(Default)]
pub struct FlappyPage {
    state: Entity<FlappyState>,
    tasks: TaskTracker,
}

impl Component for FlappyPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // Initialize state entity
        let state = cx.new_entity(FlappyState::default());
        self.state = Entity::clone(&state);

        // Observe for re-renders
        self.tasks.track(cx.observe(&self.state));

        let handle = cx.spawn_detached_task(move |_app| async move {
            use rand::Rng;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::from_entropy();

            loop {
                let (started, alive) = state.read(|s| (s.started, s.bird.alive)).unwrap_or((false, false));

                if started && alive {
                    let _ = state.update(|s| {
                        s.tick += 1;

                        // Update bird
                        s.bird.update();
                        s.bird.check_bounds(2.0, 48.0);

                        // Spawn pipes
                        if s.tick % 55 == 0 {
                            let gap_y = rng.gen_range(14.0..36.0);
                            s.pipes.push(Pipe::new(105.0, gap_y));
                        }

                        // Update pipes
                        for pipe in s.pipes.iter_mut() {
                            pipe.update();

                            if !pipe.passed && pipe.x + PIPE_WIDTH < s.bird.x {
                                pipe.passed = true;
                                s.score += 1;
                            }

                            if s.bird.collides_with_pipe(pipe.x, pipe.gap_y) {
                                s.bird.alive = false;
                            }
                        }

                        s.pipes.retain(|p| p.x > -PIPE_WIDTH);
                    });
                    // app.refresh(); // redundant with observe
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
            }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    fn render(&mut self, _cx: &mut Context<Self>) -> impl IntoElement + 'static {
        let state_data = self.state.read(|s| s.clone()).unwrap_or_default();
        let bird = state_data.bird.clone();
        let pipes = state_data.pipes.clone();
        let started = state_data.started;
        let score = state_data.score;
        let high_score = state_data.high_score;

        // Header
        let status = if !bird.alive { "GAME OVER" } else if !started { "READY" } else { "FLYING" };
        let header_color = if !bird.alive { Color::Red } else { Color::Yellow };
        let header = div()
            .h(3)
            .border_all()
            .border_type(BorderType::Rounded)
            .fg(header_color)
            .child(text(format!(
                " Score: {}  │  Best: {}  │  {} ",
                score, high_score, status
            )).bold().align_center());

        // Game canvas
        let game_view = div()
            .flex()
            .child(
                canvas(move |frame, area| {
                    let canvas_widget = RatatuiCanvas::default()
                        .block(Block::default()
                            .title(" Flappy Bird ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Cyan)))
                        .x_bounds([0.0, 100.0])
                        .y_bounds([0.0, 50.0])
                        .paint(move |ctx| {
                            // Ground
                            ctx.draw(&Rectangle { x: 0.0, y: 0.0, width: 100.0, height: 2.0, color: Color::DarkGray });

                            // Render pipes
                            for pipe in &pipes {
                                pipe.render(ctx);
                            }

                            // Render bird (particle-based)
                            bird.render(ctx);

                            // Clouds
                            ctx.print(12.0, 44.0, Line::styled("☁", Style::default().fg(Color::White)));
                            ctx.print(55.0, 46.0, Line::styled("☁", Style::default().fg(Color::White)));
                            ctx.print(85.0, 42.0, Line::styled("☁", Style::default().fg(Color::White)));

                            // Instructions
                            if !started && bird.alive {
                                ctx.print(33.0, 28.0, Line::styled("Press SPACE to fly!", Style::default().fg(Color::White)));
                            }
                            if !bird.alive {
                                ctx.print(40.0, 28.0, Line::styled("R to restart", Style::default().fg(Color::White)));
                            }
                        });
                    frame.render_widget(canvas_widget, area);
                })
            );

        // Footer
        let footer_color = if !state_data.bird.alive { Color::Red } else { Color::Yellow };
        let footer = div()
            .h(3)
            .bg(footer_color)
            .fg(Color::Black)
            .child(text(" SPACE Flap │ R Reset │ M Menu │ Q Quit ").align_center());

        // Final Layout
        div()
            .flex_col()
            .h_full()
            .child(header)
            .child(game_view)
            .child(footer)
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => Some(Action::Navigate("menu".to_string())),
                KeyCode::Char('r') => {
                    let _ = self.state.update(|s| s.reset());
                    None
                }
                KeyCode::Char(' ') | KeyCode::Up => {
                    let _ = self.state.update(|s| {
                        if !s.bird.alive {
                            s.reset();
                        }
                        if !s.started {
                            s.started = true;
                        }
                        s.bird.flap();
                    });
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}
