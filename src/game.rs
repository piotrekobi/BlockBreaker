use ggez::event::KeyCode;
use ggez::{event, graphics, timer, Context, GameResult};
use oorandom::Rand32;
use std::collections::HashMap;
use std::ops;
use std::time::Instant;

pub const GRID_SIZE: [f32; 2] = [30.0, 20.0];
pub const GRID_CELL_SIZE: [f32; 2] = [32.0, 32.0];
const DESIRED_FPS: u32 = 30;

lazy_static! {
    static ref BLOCK_TYPES: HashMap<&'static str, BlockType> = HashMap::from([
        (
            "Red",
            BlockType {
                color: (255, 0, 0).into(),
                duration: 5000,
                value: 10,
            },
        ),
        (
            "Blue",
            BlockType {
                color: (0, 0, 255).into(),
                duration: 10000,
                value: 5,
            },
        ),
        (
            "Green",
            BlockType {
                color: (0, 255, 0).into(),
                duration: 15000,
                value: 3,
            },
        ),
    ]);
}

#[derive(Clone, Copy)]
struct GridPosition {
    x: f32,
    y: f32,
}

impl GridPosition {
    pub fn random(rng: &mut Rand32, min_x: u32, max_x: u32, min_y: u32, max_y: u32) -> Self {
        (
            rng.rand_range(min_x..max_x) as f32,
            rng.rand_range(min_y..max_y) as f32,
        )
            .into()
    }
}

impl ops::Index<usize> for GridPosition {
    type Output = f32;
    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            _ => self.index(idx % 2),
        }
    }
}

impl ops::IndexMut<usize> for GridPosition {
    fn index_mut(&mut self, idx: usize) -> &mut f32 {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => self.index_mut(idx % 2),
        }
    }
}

impl From<(f32, f32)> for GridPosition {
    fn from(pos: (f32, f32)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
    None,
}

struct Ball {
    pos: GridPosition,
    speed: [f32; 2],
    offsets: [f32; 2],
}

struct Palette {
    pos: GridPosition,
    dir: Direction,
}

#[derive(Clone, Copy)]
struct BlockType {
    color: ggez::graphics::Color,
    duration: i32,
    value: i32,
}

#[derive(Clone, Copy)]
struct Block {
    pos: GridPosition,
    creation_time: Instant,
    block_type: BlockType,
    broken: bool,
    pause_time: Option<Instant>,
    prev_time_paused: i32,
    current_time_paused: i32,
}

impl Ball {
    pub fn new(pos: GridPosition) -> Self {
        Ball {
            pos: pos,
            speed: [12.0, -18.0],
            offsets: [0.0, 0.0],
        }
    }

    fn update(&mut self) {
        for i in 0..2 {
            self.offsets[i] = (self.offsets[i] + self.speed[i]).round();
            if self.offsets[i] > GRID_CELL_SIZE[i] {
                self.offsets[i] -= GRID_CELL_SIZE[i];
                self.pos[i] += 1.0;
            } else if self.offsets[i] < -GRID_CELL_SIZE[i] {
                self.offsets[i] += GRID_CELL_SIZE[i];
                self.pos[i] -= 1.0;
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [
                (self.pos.x * GRID_CELL_SIZE[0] + self.offsets[0] + (GRID_CELL_SIZE[0] / 2.0)),
                (self.pos.y * GRID_CELL_SIZE[1] + self.offsets[1] + (GRID_CELL_SIZE[1] / 2.0)),
            ],
            GRID_CELL_SIZE[0] / 2.0,
            0.1,
            (33, 67, 101).into(),
        )?;
        graphics::draw(ctx, &circle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        Ok(())
    }
}

impl Palette {
    pub fn new(pos: GridPosition) -> Self {
        Palette {
            pos: pos,
            dir: Direction::None,
        }
    }

    fn update(&mut self) {
        if self.dir == Direction::Left {
            if self.pos.x >= 3.0 {
                self.pos.x -= 1.2;
            }
        } else if self.dir == Direction::Right {
            if self.pos.x < GRID_SIZE[0] - 3.0 {
                self.pos.x += 1.2;
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new_i32(
                (self.pos.x - 2.0) as i32 * GRID_CELL_SIZE[0] as i32,
                self.pos.y as i32 * GRID_CELL_SIZE[1] as i32,
                (GRID_CELL_SIZE[0] * 5.0) as i32,
                GRID_CELL_SIZE[1] as i32,
            ),
            (101, 67, 33).into(),
        )?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        Ok(())
    }
}

impl Block {
    pub fn new(pos: GridPosition, block_type: BlockType) -> Self {
        Block {
            pos: pos,
            block_type: block_type,
            creation_time: Instant::now(),
            pause_time: None,
            prev_time_paused: 0,
            current_time_paused: 0,
            broken: false,
        }
    }

    fn draw(&mut self, ctx: &mut Context, paused: bool) -> GameResult<()> {
        if !self.broken {
            if paused {
                if self.pause_time == None {
                    self.pause_time = Instant::now().into();
                }
                self.current_time_paused = self.pause_time.unwrap().elapsed().as_millis() as i32;
            } else {
                self.pause_time = None;
                self.prev_time_paused += self.current_time_paused;
                self.current_time_paused = 0;
            }
            let total_time_paused = self.prev_time_paused + self.current_time_paused;
            let mut color = self.block_type.color;
            let transparency = (self.block_type.duration + total_time_paused
                - self.creation_time.elapsed().as_millis() as i32)
                as f32
                / self.block_type.duration as f32;
            if transparency < 0.0 {
                self.broken = true;
                return Ok(());
            }
            color.a = transparency;
            let circle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new_i32(
                    self.pos.x as i32 * GRID_CELL_SIZE[0] as i32,
                    self.pos.y as i32 * GRID_CELL_SIZE[1] as i32,
                    GRID_CELL_SIZE[0] as i32,
                    GRID_CELL_SIZE[1] as i32,
                ),
                color,
            )?;
            graphics::draw(ctx, &circle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }
        Ok(())
    }
}

fn random_block_type(rng: &mut Rand32) -> BlockType {
    let rand = rng.rand_float();
    match rand {
        x if x < 0.1 => BLOCK_TYPES["Red"],
        x if x < 0.4 => BLOCK_TYPES["Blue"],
        _ => BLOCK_TYPES["Green"],
    }
}

pub struct GameState {
    palette: Palette,
    ball: Ball,
    blocks: std::vec::Vec<Block>,
    paused: bool,
    last_block_add_time: Instant,
    random_seed: Rand32,
    score: i32,
    start_time: Instant,
    pause_time: Option<Instant>,
    time_left: i32,
    prev_time_paused: i32,
    current_time_paused: i32,
    p_pressed: bool,
}

impl GameState {
    pub fn new() -> Self {
        let palette_pos = (GRID_SIZE[0] / 2.0, GRID_SIZE[1] - 3.0).into();
        let ball_pos = (GRID_SIZE[0] / 2.0, GRID_SIZE[1] - 5.0).into();
        let mut seed: [u8; 8] = [0; 8];

        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let random_seed = Rand32::new(u64::from_ne_bytes(seed));
        let block_vec = Vec::new();

        GameState {
            palette: Palette::new(palette_pos),
            ball: Ball::new(ball_pos),
            blocks: block_vec,
            paused: true,
            last_block_add_time: Instant::now(),
            random_seed: random_seed,
            score: 0,
            start_time: Instant::now(),
            pause_time: Instant::now().into(),
            time_left: 60,
            prev_time_paused: 0,
            current_time_paused: 0,
            p_pressed: false,
        }
    }

    fn update_blocks(&mut self) {
        let mut updated_blocks = Vec::new();
        for block in &mut self.blocks {
            if !block.broken {
                updated_blocks.push(*block);
            }
        }
        self.blocks = updated_blocks;
    }

    fn set_direction(&mut self, ctx: &mut Context) {
        let keys = ggez::input::keyboard::pressed_keys(ctx);
        if keys.contains(&KeyCode::Left) & !keys.contains(&KeyCode::Right) {
            self.palette.dir = Direction::Left;
        } else if keys.contains(&KeyCode::Right) & !keys.contains(&KeyCode::Left) {
            self.palette.dir = Direction::Right;
        } else {
            self.palette.dir = Direction::None;
        }
    }

    fn set_ball_direction(&mut self) {
        if self.ball.pos.y + 1.0 >= self.palette.pos.y && self.ball.speed[1] > 0.0 {
            if (self.ball.pos.x - self.palette.pos.x).abs() < 3.3 {
                self.ball.speed[1] *= -1.0;
                self.ball.speed[0] = ((self.ball.pos.x - self.palette.pos.x) as f32 * 9.0)
                    + (self.ball.offsets[0] as f32 / (GRID_CELL_SIZE[0] / 16.0) as f32) as f32
            }
        } else if self.ball.pos.y <= 0.0 && self.ball.speed[1] < 0.0 {
            self.ball.speed[1] *= -1.0;
        } else if self.ball.pos.x <= 0.0 && self.ball.speed[0] < 0.0 {
            self.ball.speed[0] *= -1.0;
        } else if self.ball.pos.x >= GRID_SIZE[0] - 1.0 && self.ball.speed[0] > 0.0 {
            self.ball.speed[0] *= -1.0;
        }

        for block in &mut self.blocks {
            let y_abs = (self.ball.pos.y - block.pos.y).abs();
            let x_abs = (self.ball.pos.x - block.pos.x).abs();
            if x_abs < 2.0 && y_abs < 2.0 {
                if block.creation_time.elapsed().as_millis() > 100 {
                    if x_abs > y_abs {
                        self.ball.speed[0] *= -1.0;
                    } else if x_abs < y_abs {
                        self.ball.speed[1] *= -1.0;
                    } else {
                        if self.ball.offsets[0] < self.ball.offsets[1] {
                            self.ball.speed[0] *= -1.0;
                        } else {
                            self.ball.speed[1] *= -1.0;
                        }
                    }
                    block.broken = true;
                    self.score += block.block_type.value;
                }
            }
        }
    }

    fn check_restart(&mut self) {
        if self.ball.pos.y > GRID_SIZE[1] {
            self.ball.pos = (GRID_SIZE[0] / 2.0, GRID_SIZE[1] - 5.0).into();
            self.ball.speed = [12.0, -18.0];
            self.score -= 10;
            self.paused = true;
        }
    }

    fn draw_text(&self, ctx: &mut Context) -> GameResult<()> {
        let mut score_text = ggez::graphics::Text::new(format!("Score: {}", self.score));
        score_text.set_font(graphics::Font::default(), graphics::PxScale::from(40.0));
        let score_params = graphics::DrawParam::default()
            .color(graphics::Color::BLACK)
            .dest([
                GRID_CELL_SIZE[0] as f32,
                GRID_SIZE[1] as f32 * GRID_CELL_SIZE[1] as f32
                    - 1.5 * GRID_CELL_SIZE[1] as f32 as f32,
            ]);
        let mut time_text = ggez::graphics::Text::new(format!("Time left: {}", self.time_left));
        time_text.set_font(graphics::Font::default(), graphics::PxScale::from(40.0));
        let time_params = graphics::DrawParam::default()
            .color(graphics::Color::BLACK)
            .dest([
                GRID_CELL_SIZE[0] as f32 * GRID_SIZE[0] as f32 * 0.65 as f32,
                GRID_SIZE[1] as f32 * GRID_CELL_SIZE[1] as f32
                    - 1.5 * GRID_CELL_SIZE[1] as f32 as f32,
            ]);

        graphics::draw(ctx, &score_text, score_params)?;
        graphics::draw(ctx, &time_text, time_params)?;

        Ok(())
    }

    fn add_block(&mut self) {
        if self.last_block_add_time.elapsed().as_millis() > 1000 {
            self.blocks.push(Block::new(
                GridPosition::random(
                    &mut self.random_seed,
                    1,
                    (GRID_SIZE[0] - 2.0) as u32,
                    1,
                    (GRID_SIZE[1] - 10.0) as u32,
                ),
                random_block_type(&mut self.random_seed),
            ));
            self.last_block_add_time = Instant::now();
        }
    }

    fn end_game(&mut self) {
        let mut map = HashMap::new();
        map.insert("score", self.score);
        let client = reqwest::blocking::Client::new();
        let _res = client
            .post("http://127.0.0.1:5000/scores")
            .json(&map)
            .send();
        std::process::exit(0);
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            let secs = self.start_time.elapsed().as_secs() as i32
                - self.prev_time_paused
                - self.current_time_paused;
            self.time_left = 10 - secs;
            if self.time_left <= 0 {
                self.end_game();
            }
            self.set_direction(ctx);
            if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::P) {
                self.p_pressed = true
            }
            if self.p_pressed && !ggez::input::keyboard::is_key_pressed(ctx, KeyCode::P) {
                self.p_pressed = false;
                self.paused = !self.paused;
            }

            if self.paused {
                if self.pause_time == None {
                    self.pause_time = Instant::now().into();
                }
                self.current_time_paused = self.pause_time.unwrap().elapsed().as_secs() as i32;
            } else {
                if self.pause_time != None {
                    self.prev_time_paused += self.pause_time.unwrap().elapsed().as_secs() as i32;
                    self.current_time_paused = 0;
                    self.pause_time = None;
                }
            }

            if self.paused {
                if self.palette.dir != Direction::None {
                    self.paused = false;
                }
            }
            if !self.paused {
                self.palette.update();
                self.ball.update();
                self.update_blocks();
                self.set_ball_direction();
                self.add_block();
                self.check_restart();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());
        self.palette
            .draw(ctx)
            .map_err(|err| println!("{:?}", err))
            .ok();
        self.ball
            .draw(ctx)
            .map_err(|err| println!("{:?}", err))
            .ok();
        for block in &mut self.blocks {
            block
                .draw(ctx, self.paused)
                .map_err(|err| println!("{:?}", err))
                .ok();
        }
        self.draw_text(ctx)
            .map_err(|err| println!("{:?}", err))
            .ok();
        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }
}
