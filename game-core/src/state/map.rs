use std::collections::HashSet;

use crate::{
    bot::Rng,
    error::{Error, ErrorKind::*},
};

const PLAYER_DIAMETER: f32 = 0.5;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Block(pub u8, pub u8);
#[derive(Clone, Copy)]
struct Dimensions(pub u8, pub u8);

pub struct Map {
    dimensions: Dimensions,
    wall_set: HashSet<Block>,
    spawns: Vec<Block>,
    available_spawns: Vec<Block>,
    pub block_matrix: Vec<Vec<BlockKind>>,
    block_size: u32,
    /// Draws spawns. Seeded per map so a harness can replay a match; a real game seeds it
    /// from the global generator, so spawns still vary round to round.
    rng: Rng,
}

pub enum BlockKind {
    Wall,
    Ground,
}

impl Map {
    pub fn parse(map: &str, resolution: (u32, u32)) -> Map {
        Map::parse_with_seed(map, resolution, rand::random())
    }

    /// Same map, with spawn draws that repeat for a given seed.
    pub fn parse_with_seed(map: &str, resolution: (u32, u32), seed: u64) -> Map {
        let mut wall_set = HashSet::new();
        let mut spawns = Vec::new();
        let mut block_matrix = Vec::new();
        let mut y = 0;

        for row in map.lines() {
            let chars: Vec<char> = row.chars().collect();
            let mut block_row = Vec::new();

            for (x, ch) in chars.iter().enumerate() {
                match ch {
                    'x' => {
                        wall_set.insert(Block(x as u8, y as u8));
                        block_row.push(BlockKind::Wall);
                    }
                    's' => {
                        spawns.push(Block(x as u8, y as u8));
                        block_row.push(BlockKind::Ground);
                    }
                    _ => {
                        block_row.push(BlockKind::Ground);
                    }
                }
            }

            block_matrix.push(block_row);
            y += 1;
        }

        let available_spawns = spawns.clone();

        // The map file is the only source of its size: a row shorter than the widest one would
        // otherwise hold blocks that render as ground but count as wall.
        let width = block_matrix.iter().map(Vec::len).max().unwrap_or(0) as u8;
        let dimensions = Dimensions(width, block_matrix.len() as u8);
        let block_size = resolution.0 / width.max(1) as u32;

        Map {
            dimensions,
            wall_set,
            spawns,
            block_matrix,
            available_spawns,
            block_size,
            rng: Rng::new(seed),
        }
    }

    pub fn get_spawn_block(&mut self) -> Result<Block, Error> {
        let range = 0..self.available_spawns.len();
        if range.is_empty() {
            return Err(Error::new(NoSpawnAvailable, "no more spawns available"));
        }
        let random_index = (self.rng.next_u64() % range.len() as u64) as usize;
        Ok(self.available_spawns.remove(random_index))
    }

    /// Makes every spawn available again, for a new round.
    pub fn reset_spawns(&mut self) {
        self.available_spawns = self.spawns.clone();
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    /// Whether the point is in a wall. Everything outside the map counts as a wall.
    pub fn is_wall(&self, x: f64, y: f64) -> bool {
        let Dimensions(width, height) = self.dimensions;
        if x < 0. || y < 0. || x >= width as f64 || y >= height as f64 {
            return true;
        }
        self.wall_set.contains(&Block(x as u8, y as u8))
    }
}
