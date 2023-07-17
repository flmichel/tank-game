use rand::Rng;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::error::{Error, ErrorKind::*};

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
}

pub enum BlockKind {
    Wall,
    Ground,
}

impl Map {
    pub fn from_file(path: &str, resolution: (u32, u32)) -> Result<Map, Error> {
        let file = File::open(path).map_err(|err| {
            Error::from(err, ConfigurationError).explain("configuration file not found")
        })?;
        let reader = BufReader::new(file);

        let mut wall_set = HashSet::new();
        let mut spawns = Vec::new();
        let mut block_matrix = Vec::new();
        let mut y = 0;

        for line in reader.lines() {
            let row = line.map_err(|err| {
                Error::from(err, ConfigurationError).explain("could not parse line")
            })?;
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

        let dimension = Dimensions(16, 9);
        let block_size = resolution.0 / dimension.0 as u32;

        Ok(Map {
            dimensions: Dimensions(16, 9),
            wall_set,
            spawns,
            block_matrix,
            available_spawns,
            block_size,
        })
    }

    pub fn get_spawn_block(&mut self) -> Result<Block, Error> {
        let range = 0..self.available_spawns.len();
        if range.is_empty() {
            return Err(Error::new(NoSpawnAvailable, "no more spawns available"));
        }
        let random_index = rand::thread_rng().gen_range(range);
        Ok(self.available_spawns.remove(random_index))
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn is_wall(&self, block: &Block) -> bool {
        self.wall_set.contains(block)
    }
}
