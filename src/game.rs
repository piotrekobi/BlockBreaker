use ggez::event::KeyCode;
use ggez::{event, graphics, timer, Context, GameResult};
use oorandom::Rand32;
use std::ops;

pub const GRID_SIZE: [i16; 2] = [30, 20];
pub const GRID_CELL_SIZE: [i16; 2] = [32, 32];
const DESIRED_FPS: u32 = 30;

#[derive(Clone, Copy)]
struct GridPosition {
    x: i16,
    y: i16,
}

impl GridPosition {
    pub fn random(rng: &mut Rand32, max_x: i16, max_y: i16) -> Self {
        (
            rng.rand_range(0..(max_x as u32)) as i16,
            rng.rand_range(0..(max_y as u32)) as i16,
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

impl Ball {
    pub fn new(pos: GridPosition) -> Self {
        Ball {
            pos: pos,
            speed: [6.0, 6.0],
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
            [0.0, 0.0, 0.0, 1.0].into(),
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
            if self.pos.x >= 2 {
                self.pos.x -= 1;
            }
        } else if self.dir == Direction::Right {
            if self.pos.x < GRID_SIZE[0] - 2 {
                self.pos.x += 1;
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new_i32(
                (self.pos.x - 1) as i32 * GRID_CELL_SIZE[0] as i32,
                self.pos.y as i32 * GRID_CELL_SIZE[1] as i32,
                (GRID_CELL_SIZE[0] * 3) as i32,
                GRID_CELL_SIZE[1] as i32,
            ),
            [0.0, 0.0, 0.0, 1.0].into(),
        )?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        Ok(())
    }
}

pub struct GameState {
    palette: Palette,
    ball: Ball,
    gameover: bool,
    random_seed: Rand32,
}

impl GameState {
    pub fn new() -> Self {
        let palette_pos = (GRID_SIZE[0] / 2, GRID_SIZE[1] - 2).into();
        let ball_pos = (GRID_SIZE[0] / 2, 7).into();
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let random_seed = Rand32::new(u64::from_ne_bytes(seed));

        GameState {
            palette: Palette::new(palette_pos),
            ball: Ball::new(ball_pos),
            gameover: false,
            random_seed,
        }
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
            if (self.ball.pos.x - self.palette.pos.x).abs() < 2 {
                self.ball.speed[1] *= -1.0;
                self.ball.speed[0] = ((self.ball.pos.x - self.palette.pos.x) as f32 * 7.5)
                    + (self.ball.offsets[0] as f32 / (GRID_CELL_SIZE[0] / 5) as f32) as f32
            }
        } else if self.ball.pos.y == 0 && self.ball.speed[1] < 0.0 {
            self.ball.speed[1] *= -1.0;
        } else if self.ball.pos.x == 0 && self.ball.speed[0] < 0.0 {
            self.ball.speed[0] *= -1.0;
        } else if self.ball.pos.x == GRID_SIZE[0] - 1 && self.ball.speed[0] > 0.0 {
            self.ball.speed[0] *= -1.0;
        }
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.set_direction(ctx);
            if !self.gameover {
                self.palette.update();
                self.ball.update();
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
        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }
}
