use std::collections::HashSet;

use serde::{Serialize, Deserialize};

use crate::rand;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}
impl Direction {
    pub fn id(self) -> &'static str {
        match self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right"
        }
    }
}
pub type RobotPositions = [usize; 5];

#[derive(Clone, Serialize, Deserialize)]
pub struct Board {
    pub width: usize,

    /// A width * (height - 1) size array.
    /// There is a wall between (i, j) and (i, j+1) iff
    /// horizontal_walls[i, j] is true.
    pub horizontal_walls: Vec<bool>,

    /// A (width - 1) * height size array.
    /// There is a wall between (i, j) and (i+1, j) iff
    /// verticall_walls[i, j] is true.
    pub vertical_walls: Vec<bool>,

    // Initial position of the robots
    pub initial_positions: RobotPositions
}

pub const RED: usize = 0;
pub const YELLOW: usize = 1;
pub const GREEN: usize = 2;
pub const BLUE: usize = 3;
pub const BLACK: usize = 4;

impl Board {
    pub fn height(&self) -> usize {
        self.horizontal_walls.len() / self.width + 1
    }

    pub fn generate(width: usize, height: usize) -> Self {
        let mut board = Board {
            width,
            horizontal_walls: vec![false; width * (height - 1)],
            vertical_walls: vec![false; (width - 1) * height],
            initial_positions: [0, 1, 2, 3, 4]
        };
        

        let mut used_tiles: HashSet<(usize, usize)> = HashSet::new();

        for _ in 0..16 {
            let mut i: usize = 0;
            let mut j: usize = 0;
            loop {
                i = rand::uniform(1, board.width - 1);
                j = rand::uniform(1, board.height() - 1);
                if false
                    || used_tiles.contains(&(i+1, j-1))
                    || used_tiles.contains(&(i+1, j))
                    || used_tiles.contains(&(i+1, j+1))
                    || used_tiles.contains(&(i, j-1))
                    || used_tiles.contains(&(i, j))
                    || used_tiles.contains(&(i, j+1))
                    || used_tiles.contains(&(i-1, j-1))
                    || used_tiles.contains(&(i-1, j))
                    || used_tiles.contains(&(i-1, j+1)) {
                    continue;
                }
                break;
            }
            used_tiles.insert((i, j));

            board.horizontal_walls[(j + rand::uniform(0, 2) - 1)*width+i] = true;
            board.vertical_walls[j*(width-1)+i+ rand::uniform(0, 2) - 1] = true;    
        }

        board.horizontal_walls[rand::uniform(4, 7)*width] = true;
        board.horizontal_walls[rand::uniform(9, 12)*width] = true;
        board.horizontal_walls[rand::uniform(4, 7)*width+(width-1)] = true;
        board.horizontal_walls[rand::uniform(9, 12)*width+(width-1)] = true;

        board.vertical_walls[rand::uniform(4, 7)] = true;
        board.vertical_walls[rand::uniform(9, 12)] = true;
        board.vertical_walls[rand::uniform(4, 7)+(height-1)*(width-1)] = true;
        board.vertical_walls[rand::uniform(9, 12)+(height-1)*(width-1)] = true;

        // block off center tiles
        board.vertical_walls[width/2 - 2 + (height/2-1)*(width - 1)] = true;
        board.vertical_walls[width/2 + 0 + (height/2-1)*(width - 1)] = true;
        board.vertical_walls[width/2 - 2 + (height/2)*(width - 1)] = true;
        board.vertical_walls[width/2 + 0 + (height/2)*(width - 1)] = true;
        board.horizontal_walls[width/2 - 1 + (height/2 - 2)*width] = true;
        board.horizontal_walls[width/2 + 0 + (height/2 - 2)*width] = true;
        board.horizontal_walls[width/2 - 1 + (height/2 + 0)*width] = true;
        board.horizontal_walls[width/2 + 0 + (height/2 + 0)*width] = true;

        board
    }

    /// Returns whether the given tile index
    /// represents a center (blocked-off) tile.
    pub fn is_center_tile(&self, tile: usize) -> bool {
        let x = tile % self.width;
        let y = tile / self.width;
        let height = self.height();

        self.width / 2 - 1 <= x && x <= self.width / 2 &&
            height / 2 - 1 <= y && y <= height / 2
    }

    /// Given a robot position, move a robot in
    /// a direction. Returns the new robot positions.
    pub fn move_robot(&self, mut positions: RobotPositions, robot: usize, direction: Direction) -> RobotPositions {
        let width = self.width;
        let height = self.height();
        match direction {
            Direction::Down => {
                let wall_pos = self.horizontal_walls.iter().enumerate()
                    .filter(|(i, b)| (**b) && (*i % width  == positions[robot] % width))
                    .map(|(i, _)| i / width)
                    .filter(|&i| i + 1 > positions[robot] / width)
                    .next().unwrap_or(height - 1);
                let robot_pos = positions.iter()
                    .filter(|&i| i % width == positions[robot] % width)
                    .map(|i| i / width)
                    .filter(|&i| i > positions[robot] / width)
                    .min().map(|i| i - 1).unwrap_or(height - 1);
                let target_pos = usize::min(wall_pos, robot_pos);

                positions[robot] = positions[robot] % width + target_pos * width;
                positions   
            },
            Direction::Up => {
                let wall_pos = self.horizontal_walls.iter().enumerate().rev()
                    .filter(|(i, b)| (**b) && (*i % width  == positions[robot] % width))
                    .map(|(i, _)| i / width)
                    .filter(|&i| i < positions[robot] / width)
                    .next().map(|i| i + 1).unwrap_or(0);
                let robot_pos = positions.iter()
                    .filter(|&i| i % width == positions[robot] % width)
                    .map(|i| i / width)
                    .filter(|&i| i < positions[robot] / width)
                    .max().map(|i| i + 1).unwrap_or(0);
                let target_pos = usize::max(wall_pos, robot_pos);

                positions[robot] = positions[robot] % width + target_pos * width;
                positions
            },
            Direction::Right => {
                let wall_pos = self.vertical_walls.iter().enumerate()
                    .filter(|&(i, b)| (*b && i / (width - 1) == positions[robot] / width))
                    .map(|(i, _)| i % (width - 1))
                    .filter(|&i| i + 1 > positions[robot] % width)
                    .next().map(|i| i).unwrap_or(width - 1);
                let robot_pos = positions.iter()
                    .filter(|&i| i / width == positions[robot] / width)
                    .map(|i| i % width)
                    .filter(|&i| i > positions[robot] % width)
                    .min().map(|i| i - 1).unwrap_or(width - 1);
                let target_pos = usize::min(wall_pos, robot_pos);

                positions[robot] = (positions[robot] / width) * width + target_pos;
                positions
            },
            Direction::Left => {
                let wall_pos = self.vertical_walls.iter().enumerate().rev()
                    .filter(|&(i, b)| *b && i / (width - 1) == positions[robot] / width)
                    .map(|(i, _)| i % (width - 1))
                    .filter(|&i| i < positions[robot] % width)
                    .next().map(|i| i + 1).unwrap_or(0);
                let robot_pos = positions.iter()
                    .filter(|&i| i / width == positions[robot] / width)
                    .map(|i| i % width)
                    .filter(|&i| i < positions[robot] % width)
                    .max().map(|i| i + 1).unwrap_or(0);
                let target_pos = usize::max(wall_pos, robot_pos);

                positions[robot] = (positions[robot] / width) * width + target_pos;
                positions
            }
        }
    }
}