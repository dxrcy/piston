use std::fmt;
use std::io::{self, Write as _};
use std::ops;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, terminal};

fn main() -> io::Result<()> {
    let mut grid = Grid::new(10, 10);
    grid[(5, 3)] = Tile::Piston(Direction::Right);
    grid[(6, 3)] = Tile::Stone;
    grid[(7, 3)] = Tile::Stone;
    grid[(8, 3)] = Tile::Stone;
    // grid[(8, 3)] = Tile::Block;
    // grid[(7, 3)] = Tile::PistonArm(Direction::Up);

    grid[(2, 4)] = Tile::Piston(Direction::Down);
    grid[(2, 5)] = Tile::Stone;

    grid[(3, 5)] = Tile::Bedrock;

    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    loop {
        grid.draw(&mut stdout)?;
        grid.tick();

        match event::read()? {
            Event::Key(event) => match (event.modifiers, event.code) {
                (KeyModifiers::NONE, KeyCode::Char('h')) => {
                    //
                }
                (KeyModifiers::NONE, KeyCode::Char('q'))
                | (KeyModifiers::NONE, KeyCode::Esc)
                | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    break;
                }
                _ => (),
            },
            _ => (),
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

#[derive(Debug)]
struct Grid {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: [Tile::Empty].repeat(width * height),
        }
    }

    fn contains(&self, coord: Coord) -> bool {
        coord.x >= 0
            && coord.y >= 0
            && (coord.x as usize) < self.width
            && (coord.y as usize) < self.height
    }

    fn find_push_end(&self, start: Coord, direction: Direction) -> Option<Coord> {
        let mut end = start;
        loop {
            end += Coord::from_direction(1, direction);
            if !self.contains(end) {
                return None;
            }
            if !self[end].is_movable() {
                return None;
            }
            if self[end].is_empty() {
                return Some(end);
            }
        }
    }

    pub fn tick(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = Coord::from((x, y));

                match self[(x, y)] {
                    Tile::Piston(direction) => {
                        let Some(end) = self.find_push_end(coord, direction) else {
                            continue;
                        };
                        assert!(coord.shares_axis(end));

                        let mut inner = end;
                        while inner != coord {
                            let outer = inner;
                            inner += Coord::from_direction(1, direction.flip());
                            self[outer] = self[inner];
                        }

                        self[coord] = Tile::PistonBase(direction);
                        self[coord.add_direction(1, direction)] = Tile::PistonArm(direction);
                    }

                    Tile::PistonBase(direction) => {
                        let arm_coord = coord.add_direction(1, direction);
                        assert_eq!(self[arm_coord], Tile::PistonArm(direction));

                        self[coord] = Tile::Piston(direction);

                        // Sticky
                        let pull_coord = coord.add_direction(2, direction);
                        if self.contains(pull_coord) && self[pull_coord].is_movable() {
                            self[arm_coord] = self[pull_coord];
                            self[pull_coord] = Tile::Empty;
                        } else {
                            self[arm_coord] = Tile::Empty;
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    pub fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        let origin = Coord::from((1, 1));

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(origin.x as u16, origin.y as u16),
        )?;
        for _ in 0..self.width + 2 {
            print!("{}", Tile::Bedrock);
        }

        for y in 0..self.height {
            execute!(
                stdout,
                cursor::MoveTo(origin.x as u16, origin.y as u16 + y as u16 + 1)
            )?;

            print!("{}", Tile::Bedrock);
            for x in 0..self.width {
                print!("{}", self[(x, y)]);
            }
            print!("{}", Tile::Bedrock);
        }

        execute!(
            stdout,
            cursor::MoveTo(origin.x as u16, origin.y as u16 + self.height as u16 + 1),
        )?;
        for _ in 0..self.width + 2 {
            print!("{}", Tile::Bedrock);
        }

        println!("\r"); // Also flushes

        Ok(())
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.width && x < self.height);
        let index = x + y * self.width;
        assert!(self.tiles.len() > index);
        index
    }
}

impl ops::Index<Coord> for Grid {
    type Output = Tile;
    fn index(&self, index: Coord) -> &Self::Output {
        assert!(index.x >= 0 && index.y >= 0);
        let index = self.get_index(index.x as usize, index.y as usize);
        &self.tiles[index]
    }
}
impl ops::IndexMut<Coord> for Grid {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        assert!(index.x >= 0 && index.y >= 0);
        let index = self.get_index(index.x as usize, index.y as usize);
        &mut self.tiles[index]
    }
}
impl ops::Index<(usize, usize)> for Grid {
    type Output = Tile;
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        let index = self.get_index(x, y);
        &self.tiles[index]
    }
}
impl ops::IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let index = self.get_index(x, y);
        &mut self.tiles[index]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Empty,
    Stone,
    Bedrock,
    Piston(Direction),
    PistonBase(Direction),
    PistonArm(Direction),
}

impl Tile {
    pub fn is_empty(self) -> bool {
        matches!(self, Tile::Empty)
    }

    pub fn is_movable(self) -> bool {
        !matches!(self, Tile::PistonBase(_) | Tile::PistonArm(_))
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Empty => (),
            Tile::Stone => write!(f, "\x1b[97;101m")?,
            Tile::Bedrock => write!(f, "\x1b[97;40m")?,
            Tile::Piston(_) => write!(f, "\x1b[33;44m")?,
            Tile::PistonBase(_) => write!(f, "\x1b[33;103m")?,
            Tile::PistonArm(_) => write!(f, "\x1b[34m")?,
        }

        match self {
            Tile::Empty => write!(f, "  ")?,
            Tile::Stone => write!(f, "🬴🬸")?,
            Tile::Bedrock => write!(f, "🬗🬔")?,

            Tile::Piston(Direction::Up) => write!(f, "pu")?,
            Tile::Piston(Direction::Right) => write!(f, "█▍")?,
            Tile::Piston(Direction::Down) => write!(f, "🮄🮄")?,
            Tile::Piston(Direction::Left) => write!(f, "pl")?,

            Tile::PistonBase(Direction::Up) => write!(f, "bu")?,
            Tile::PistonBase(Direction::Right) => write!(f, "█▍")?,
            Tile::PistonBase(Direction::Down) => write!(f, "🮅🮅")?,
            Tile::PistonBase(Direction::Left) => write!(f, "bl")?,

            Tile::PistonArm(Direction::Up) => write!(f, "au")?,
            Tile::PistonArm(Direction::Right) => write!(f, "🬋🬫")?,
            Tile::PistonArm(Direction::Down) => write!(f, "🬷🬲")?,
            Tile::PistonArm(Direction::Left) => write!(f, "al")?,
        }

        match self {
            Tile::Empty => (),
            Tile::Stone
            | Tile::Bedrock
            | Tile::Piston(_)
            | Tile::PistonBase(_)
            | Tile::PistonArm(_) => write!(f, "\x1b[0m")?,
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn flip(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

type CoordValue = i32;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Coord {
    x: CoordValue,
    y: CoordValue,
}

impl Coord {
    pub fn from_direction(value: CoordValue, direction: Direction) -> Self {
        match direction {
            Direction::Up => (0, -value).into(),
            Direction::Right => (value, 0).into(),
            Direction::Down => (0, value).into(),
            Direction::Left => (-value, 0).into(),
        }
    }

    pub fn add_direction(self, value: CoordValue, direction: Direction) -> Self {
        self + Self::from_direction(value, direction)
    }

    pub fn shares_axis(self, other: Self) -> bool {
        self.x == other.x || self.y == other.y
    }
}

impl ops::Add<Coord> for Coord {
    type Output = Self;
    fn add(self, rhs: Coord) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl ops::AddAssign<Coord> for Coord {
    fn add_assign(&mut self, rhs: Coord) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl From<(i32, i32)> for Coord {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            x: x as CoordValue,
            y: y as CoordValue,
        }
    }
}
impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Self {
        Self {
            x: x as CoordValue,
            y: y as CoordValue,
        }
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
