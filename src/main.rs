extern crate sdl2;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use std::time::Duration;

// Constants for Screen Size, etc
const GRID_X_SIZE: i32 = 40;
const GRID_Y_SIZE: i32 = 30;
const GRID_ZERO: i32 = 0;
const DOT_SIZE_IN_PXS: i32 = 20;
const FRAMES_PER_SECOND: u32 = 60;

// Enums
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}
pub enum PlayerDirection {
    Up,
    Down,
    Left,
    Right,
}

// Structs
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Point(pub i32, pub i32);

pub struct GameContext {
    pub player_position: Vec<Point>,
    pub player_direction: PlayerDirection,
    pub food: Point,
    pub state: GameState,
}

pub struct Renderer {
    canvas: WindowCanvas,
}

// Implementations
impl std::ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

// Game Logic. Only one context allowed.
impl GameContext {
    pub fn new() -> GameContext {
        GameContext {
            player_position: vec![Point(3, 1), Point(2, 1), Point(1, 1)],
            player_direction: PlayerDirection::Right,
            state: GameState::Paused,
            food: Point(3, 3),
        }
    }

    pub fn next_tick(&mut self) {
        if let GameState::Paused | GameState::GameOver = self.state {
            return;
        }

        let head_position = self.player_position.first().unwrap();
	
	// The body is just everything except the head.
        let body = &self.player_position[2..self.player_position.len()];

        let next_head_position = match self.player_direction {
            PlayerDirection::Up => *head_position + Point(0, -1),
            PlayerDirection::Down => *head_position + Point(0, 1),
            PlayerDirection::Right => *head_position + Point(1, 0),
            PlayerDirection::Left => *head_position + Point(-1, 0),
        };

	// If we are out of bounds or we touch our body, end the game
        if (next_head_position.0 < GRID_ZERO)
            || next_head_position.0 >= GRID_X_SIZE
            || next_head_position.1 < GRID_ZERO
            || next_head_position.1 >= GRID_Y_SIZE
            || body.contains(&head_position)  {
            self.state = GameState::GameOver;
            return;
        }

	// If we grab food, grow and and reset the food. 
        if self.food != next_head_position {
            self.player_position.pop();
        } else {
            self.spawn_food()
        }

        self.player_position.reverse();
        self.player_position.push(next_head_position);
        self.player_position.reverse();
    }

    pub fn move_up(&mut self) {
        self.player_direction = PlayerDirection::Up;
    }

    pub fn move_down(&mut self) {
        self.player_direction = PlayerDirection::Down;
    }

    pub fn move_right(&mut self) {
        self.player_direction = PlayerDirection::Right;
    }

    pub fn move_left(&mut self) {
        self.player_direction = PlayerDirection::Left;
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::GameOver => GameState::GameOver,
        }
    }

    // Spawn the food at a random location
    // TODO: Implement SDL2's rand function instead
    fn spawn_food(&mut self) {
        let mut rand = rand::thread_rng();
        self.food = Point(rand.gen_range(0..GRID_X_SIZE), rand.gen_range(0..GRID_Y_SIZE));
    }
}

// Rendering each pixel.
impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas })
    }

    fn draw_dot(&mut self, point: &Point) -> Result<(), String> {
        let Point(x, y) = point;
        self.canvas.fill_rect(Rect::new(
            x * DOT_SIZE_IN_PXS as i32,
            y * DOT_SIZE_IN_PXS as i32,
            DOT_SIZE_IN_PXS.try_into().unwrap(),
            DOT_SIZE_IN_PXS.try_into().unwrap(),
        ))?;

        Ok(())
    }

    pub fn draw(&mut self, context: &GameContext) -> Result<(), String> {
        self.draw_background(context);
        self.draw_food(context)?;
        self.draw_player(context)?;
        self.canvas.present();

        Ok(())
    }

    fn draw_background(&mut self, context: &GameContext) {
        let color = match context.state {
            GameState::Playing => Color::RGB(0, 0, 0),
            GameState::Paused => Color::RGB(30, 30, 30),
            GameState::GameOver => Color::RED,
        };
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    fn draw_player(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::GREEN);
        for point in &context.player_position {
            self.draw_dot(point)?;
        }

        Ok(())
    }

    fn draw_food(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RED);
        self.draw_dot(&context.food)?;
        Ok(())
    }
}

// Main Game Loops
pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Rust Snake Game",
            (GRID_X_SIZE * DOT_SIZE_IN_PXS).try_into().unwrap(),
            (GRID_Y_SIZE * DOT_SIZE_IN_PXS).try_into().unwrap(),
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut renderer = Renderer::new(window)?;
    let mut context = GameContext::new();
    renderer.draw(&context)?;
    let mut frame_counter = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W => context.move_up(),
                    Keycode::A => context.move_left(),
                    Keycode::S => context.move_down(),
                    Keycode::D => context.move_right(),
                    Keycode::Escape => context.toggle_pause(),
                    Keycode::Space => context = GameContext::new(),
                    _ => {}
                },
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMES_PER_SECOND));
        // The rest of the game loop goes here...

        frame_counter += 1;
        if frame_counter % 5 == 0 {
            context.next_tick();
            frame_counter = 0;
        }

        renderer.draw(&context)?;
    }

    Ok(())
}
