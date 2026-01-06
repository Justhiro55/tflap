use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const BIRD_X: u16 = 10;
const GRAVITY: f32 = 0.3;
const JUMP_VELOCITY: f32 = -1.5;
const PIPE_WIDTH: u16 = 6;
const PIPE_GAP: u16 = 8;
const PIPE_SPEED: u16 = 1;
const TICK_RATE: Duration = Duration::from_millis(50);

#[derive(PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

struct Bird {
    y: f32,
    velocity: f32,
}

impl Bird {
    fn new(y: f32) -> Self {
        Self { y, velocity: 0.0 }
    }

    fn jump(&mut self) {
        self.velocity = JUMP_VELOCITY;
    }

    fn update(&mut self) {
        self.velocity += GRAVITY;
        self.y += self.velocity;
    }

    fn reset(&mut self, y: f32) {
        self.y = y;
        self.velocity = 0.0;
    }
}

struct Pipe {
    x: i32,
    gap_y: u16,
    passed: bool,
}

impl Pipe {
    fn new(x: i32, gap_y: u16) -> Self {
        Self {
            x,
            gap_y,
            passed: false,
        }
    }

    fn update(&mut self) {
        self.x -= PIPE_SPEED as i32;
    }

    fn is_offscreen(&self) -> bool {
        self.x + PIPE_WIDTH as i32 <= 0
    }

    fn collides_with(&self, bird_x: u16, bird_y: u16) -> bool {
        let bird_x = bird_x as i32;
        if bird_x + 2 > self.x
            && bird_x < self.x + PIPE_WIDTH as i32
            && (bird_y < self.gap_y || bird_y >= self.gap_y + PIPE_GAP)
        {
            return true;
        }
        false
    }

    fn has_bird_passed(&self, bird_x: u16) -> bool {
        bird_x as i32 > self.x + PIPE_WIDTH as i32
    }
}

fn get_highscore_path() -> Option<PathBuf> {
    env::var("HOME").ok().map(|home| {
        let mut path = PathBuf::from(home);
        path.push(".tflap_highscore");
        path
    })
}

fn load_highscore() -> u32 {
    if let Some(path) = get_highscore_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            return content.trim().parse().unwrap_or(0);
        }
    }
    0
}

fn save_highscore(score: u32) {
    if let Some(path) = get_highscore_path() {
        let _ = fs::write(&path, score.to_string());
    }
}

struct Game {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: u32,
    high_score: u32,
    is_new_record: bool,
    state: GameState,
    width: u16,
    height: u16,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        let mut game = Self {
            bird: Bird::new((height / 2) as f32),
            pipes: Vec::new(),
            score: 0,
            high_score: load_highscore(),
            is_new_record: false,
            state: GameState::Playing,
            width,
            height,
        };

        // Spawn initial pipes spread across the screen
        let mut rng = rand::thread_rng();
        for i in 0..4 {
            let min_gap_y = 3;
            let max_gap_y = game.height.saturating_sub(PIPE_GAP + 3);
            let gap_y = rng.gen_range(min_gap_y..=max_gap_y);
            let x = width as i32 / 2 + (i * 40);
            game.pipes.push(Pipe::new(x, gap_y));
        }

        game
    }

    fn spawn_pipe(&mut self) {
        let mut rng = rand::thread_rng();
        let min_gap_y = 3;
        let max_gap_y = self.height.saturating_sub(PIPE_GAP + 3);
        let gap_y = rng.gen_range(min_gap_y..=max_gap_y);

        // Calculate next pipe position - always 40 pixels after the last pipe
        let new_x = if let Some(last_pipe) = self.pipes.last() {
            last_pipe.x + 40
        } else {
            self.width as i32
        };

        self.pipes.push(Pipe::new(new_x, gap_y));
    }

    fn update(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        self.bird.update();

        // Check boundary collision
        if self.bird.y < 0.0 || self.bird.y as u16 >= self.height {
            self.state = GameState::GameOver;
            self.check_and_save_highscore();
            return;
        }

        // Update pipes and check for scoring
        let bird_y = self.bird.y as u16;
        for pipe in &mut self.pipes {
            pipe.update();

            // Check if bird passed this pipe
            if !pipe.passed && pipe.has_bird_passed(BIRD_X) {
                pipe.passed = true;
                self.score += 1;
            }
        }

        // Check pipe collision
        for pipe in &self.pipes {
            if pipe.collides_with(BIRD_X, bird_y) {
                self.state = GameState::GameOver;
                self.check_and_save_highscore();
                return;
            }
        }

        // Remove offscreen pipes
        self.pipes.retain(|pipe| !pipe.is_offscreen());

        // Spawn new pipe if the rightmost pipe has moved into view
        if let Some(last_pipe) = self.pipes.last() {
            if last_pipe.x < self.width as i32 - 20 {
                self.spawn_pipe();
            }
        } else {
            // If no pipes, spawn one at the right edge
            let mut rng = rand::thread_rng();
            let min_gap_y = 3;
            let max_gap_y = self.height.saturating_sub(PIPE_GAP + 3);
            let gap_y = rng.gen_range(min_gap_y..=max_gap_y);
            self.pipes.push(Pipe::new(self.width as i32, gap_y));
        }
    }

    fn jump(&mut self) {
        if self.state == GameState::Playing {
            self.bird.jump();
        }
    }

    fn check_and_save_highscore(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
            self.is_new_record = true;
            save_highscore(self.high_score);
        }
    }

    fn reset(&mut self) {
        self.bird.reset((self.height / 2) as f32);
        self.pipes.clear();
        self.score = 0;
        self.is_new_record = false;
        self.state = GameState::Playing;

        // Spawn initial pipes spread across the screen
        let mut rng = rand::thread_rng();
        for i in 0..4 {
            let min_gap_y = 3;
            let max_gap_y = self.height.saturating_sub(PIPE_GAP + 3);
            let gap_y = rng.gen_range(min_gap_y..=max_gap_y);
            let x = self.width as i32 / 2 + (i * 40);
            self.pipes.push(Pipe::new(x, gap_y));
        }
    }

    fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(stdout, Clear(ClearType::All))?;

        // Draw pipes
        execute!(stdout, SetForegroundColor(Color::Green))?;
        for pipe in &self.pipes {
            // Skip drawing if pipe is completely off screen
            if pipe.x + PIPE_WIDTH as i32 <= 0 || pipe.x >= self.width as i32 {
                continue;
            }

            // Only draw if x is positive
            if pipe.x >= 0 {
                let pipe_x = pipe.x as u16;
                // Draw top pipe
                for y in 0..pipe.gap_y {
                    execute!(
                        stdout,
                        MoveTo(pipe_x, y),
                        Print("█".repeat(PIPE_WIDTH as usize))
                    )?;
                }
                // Draw bottom pipe
                for y in (pipe.gap_y + PIPE_GAP)..self.height {
                    execute!(
                        stdout,
                        MoveTo(pipe_x, y),
                        Print("█".repeat(PIPE_WIDTH as usize))
                    )?;
                }
            }
        }

        // Draw bird
        execute!(stdout, SetForegroundColor(Color::Yellow))?;
        let bird_y = self.bird.y as u16;
        if bird_y < self.height {
            execute!(stdout, MoveTo(BIRD_X, bird_y), Print("@"))?;
        }

        // Draw score
        execute!(stdout, SetForegroundColor(Color::Cyan))?;
        execute!(
            stdout,
            MoveTo(2, self.height - 1),
            Print(format!(
                "Score: {}  High Score: {}",
                self.score, self.high_score
            ))
        )?;

        // Draw game over screen
        if self.state == GameState::GameOver {
            let msg_y = self.height / 2;
            let msg_x = self.width / 2 - 12;

            if self.is_new_record {
                execute!(stdout, SetForegroundColor(Color::Yellow))?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y - 1),
                    Print("╔══════════════════════════╗")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y),
                    Print("║   *** NEW RECORD! ***    ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 1),
                    Print(format!("║   Score: {:5}            ║", self.score))
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 2),
                    Print("║                          ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 3),
                    Print("║   R: Retry               ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 4),
                    Print("║   Q: Quit                ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 5),
                    Print("╚══════════════════════════╝")
                )?;
            } else {
                execute!(stdout, SetForegroundColor(Color::Red))?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y - 1),
                    Print("╔══════════════════════════╗")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y),
                    Print("║   GAME OVER!             ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 1),
                    Print(format!("║   Score: {:5}            ║", self.score))
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 2),
                    Print(format!("║   Best:  {:5}            ║", self.high_score))
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 3),
                    Print("║                          ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 4),
                    Print("║   R: Retry               ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 5),
                    Print("║   Q: Quit                ║")
                )?;
                execute!(
                    stdout,
                    MoveTo(msg_x, msg_y + 6),
                    Print("╚══════════════════════════╝")
                )?;
            }
        }

        execute!(stdout, ResetColor)?;
        stdout.flush()?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Setup terminal
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let (width, height) = terminal::size()?;
    let mut game = Game::new(width, height);
    let mut last_tick = Instant::now();

    let result = run_game(&mut stdout, &mut game, &mut last_tick);

    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn run_game(stdout: &mut io::Stdout, game: &mut Game, last_tick: &mut Instant) -> io::Result<()> {
    loop {
        game.draw(stdout)?;

        // Handle input - process all pending events
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Char('c') | KeyCode::Char('C')
                        if modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        return Ok(());
                    }
                    KeyCode::Char(' ') => {
                        if game.state == GameState::Playing {
                            game.jump();
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if game.state == GameState::GameOver {
                            game.reset();
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // Update game state
        if last_tick.elapsed() >= TICK_RATE {
            game.update();
            *last_tick = Instant::now();
        }

        // Small sleep to prevent busy waiting
        std::thread::sleep(Duration::from_millis(5));
    }
}
