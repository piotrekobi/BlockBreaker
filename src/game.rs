use ggez::event::KeyCode;
use ggez::{event, graphics, timer, Context, GameResult};
use oorandom::Rand32;
use std::collections::HashMap;
use std::ops;
use std::time::Instant;

pub const GRID_SIZE: [i16; 2] = [30, 20];
pub const GRID_CELL_SIZE: [i16; 2] = [32, 32];
const DESIRED_FPS: u32 = 30;

lazy_static! {
    static ref BLOCK_TYPES: HashMap<&'static str, BlockType> = HashMap::from(
        [
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
        ]
        .iter()
        .cloned()
        .collect(),
    );
}

#[derive(Clone, Copy)]
struct GridPosition {
    x: i16,
    y: i16,
}

impl GridPosition {
    pub fn random(rng: &mut Rand32, min_x: u32, max_x: u32, min_y: u32, max_y: u32) -> Self {
        (
            rng.rand_range(min_x..max_x) as i16,
            rng.rand_range(min_y..max_y) as i16,
        )
            .into()
    }
}

impl ops::Index<usize> for GridPosition {
    type Output = i16;
    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            _ => self.index(idx % 2),
        }
    }
}

impl ops::IndexMut<usize> for GridPosition {
    fn index_mut(&mut self, idx: usize) -> &mut i16 {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => self.index_mut(idx % 2),
        }
    }
}

impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
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
    offsets: [i16; 2],
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
}

impl Ball {
    pub fn new(pos: GridPosition) -> Self {
        Ball {
            pos: pos,
            speed: [12.0, -12.0],
            offsets: [0, 0],
        }
    }

    fn update(&mut self) {
        for i in 0..2 {
            self.offsets[i] = (self.offsets[i] as f32 + self.speed[i]).round() as i16;
            if self.offsets[i] > GRID_CELL_SIZE[i] {
                self.offsets[i] -= GRID_CELL_SIZE[i];
                self.pos[i] += 1;
            } else if self.offsets[i] < -GRID_CELL_SIZE[i] {
                self.offsets[i] += GRID_CELL_SIZE[i];
                self.pos[i] -= 1;
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [
                (self.pos.x * GRID_CELL_SIZE[0] + self.offsets[0] + (GRID_CELL_SIZE[0] / 2)) as f32,
                (self.pos.y * GRID_CELL_SIZE[1] + self.offsets[1] + (GRID_CELL_SIZE[1] / 2)) as f32,
            ],
            (GRID_CELL_SIZE[0] / 2) as f32,
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
            if self.pos.x >= 3 {
                self.pos.x -= 1;
            }
        } else if self.dir == Direction::Right {
            if self.pos.x < GRID_SIZE[0] - 3 {
                self.pos.x += 1;
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new_i32(
                (self.pos.x - 2) as i32 * GRID_CELL_SIZE[0] as i32,
                self.pos.y as i32 * GRID_CELL_SIZE[1] as i32,
                (GRID_CELL_SIZE[0] * 5) as i32,
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
            broken: false,
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.broken {
            let transparency =
                (self.block_type.duration - self.creation_time.elapsed().as_millis() as i32) as f32
                    / self.block_type.duration as f32;
            if transparency < 0.0 {
                self.broken = true;
                return Ok(());
            }
            let mut color = self.block_type.color;
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
}

impl GameState {
    pub fn new() -> Self {
        let palette_pos = (GRID_SIZE[0] / 2, GRID_SIZE[1] - 3).into();
        let ball_pos = (GRID_SIZE[0] / 2, GRID_SIZE[1] - 5).into();
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
        if self.ball.pos.y + 1 == self.palette.pos.y && self.ball.speed[1] > 0.0 {
            if (self.ball.pos.x - self.palette.pos.x).abs() < 3 {
                self.ball.speed[1] *= -1.0;
                self.ball.speed[0] = ((self.ball.pos.x - self.palette.pos.x) as f32 * 9.0)
                    + (self.ball.offsets[0] as f32 / (GRID_CELL_SIZE[0] / 16) as f32) as f32
            }
        } else if self.ball.pos.y == 0 && self.ball.speed[1] < 0.0 {
            self.ball.speed[1] *= -1.0;
        } else if self.ball.pos.x == 0 && self.ball.speed[0] < 0.0 {
            self.ball.speed[0] *= -1.0;
        } else if self.ball.pos.x == GRID_SIZE[0] - 1 && self.ball.speed[0] > 0.0 {
            self.ball.speed[0] *= -1.0;
        }

        for block in &mut self.blocks {
            let y_abs = (self.ball.pos.y - block.pos.y).abs();
            let x_abs = (self.ball.pos.x - block.pos.x).abs();
            if x_abs < 2 && y_abs < 2 {
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

    fn draw_score(&self, ctx: &mut Context) -> GameResult<()> {
        let mut score_text = ggez::graphics::Text::new(format!("Score: {}", self.score));
        score_text.set_font(graphics::Font::default(), graphics::PxScale::from(40.0));

        let params = graphics::DrawParam::default()
            .color(graphics::Color::BLACK)
            .dest([
                GRID_CELL_SIZE[0] as f32,
                GRID_SIZE[1] as f32 * GRID_CELL_SIZE[1] as f32
                    - 1.5 * GRID_CELL_SIZE[1] as f32 as f32,
            ]);
        graphics::draw(ctx, &score_text, params)?;
        Ok(())
    }

    fn add_block(&mut self) {
        if self.last_block_add_time.elapsed().as_millis() > 1000 {
            self.blocks.push(Block::new(
                GridPosition::random(
                    &mut self.random_seed,
                    1,
                    (GRID_SIZE[0] - 2) as u32,
                    1,
                    (GRID_SIZE[1] - 10) as u32,
                ),
                random_block_type(&mut self.random_seed),
            ));
            self.last_block_add_time = Instant::now();
        }
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.set_direction(ctx);
            if self.palette.dir != Direction::None {
                self.paused = false;
            }
            self.add_block();
            if !self.paused {
                self.palette.update();
                self.ball.update();
                self.update_blocks();
                self.set_ball_direction();
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
            block.draw(ctx).map_err(|err| println!("{:?}", err)).ok();
        }
        self.draw_score(ctx)
            .map_err(|err| println!("{:?}", err))
            .ok();
        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }
}
