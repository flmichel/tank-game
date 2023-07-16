use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::error::{Error, ErrorKind::*};

const PLAYER_DIAMETER: f32 = 0.5;

#[derive(PartialEq, Eq, Hash)]
struct Block(i8, i8);

pub struct Map {
    wall_set: HashSet<Block>,
    spawns: Vec<Block>,
    pub block_matrix: Vec<Vec<BlockKind>>,
}

pub enum BlockKind {
    Wall,
    Ground,
}

impl Map {
    pub fn from_file(path: &str) -> Result<Map, Error> {
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
                        wall_set.insert(Block(x as i8, y as i8));
                        block_row.push(BlockKind::Wall);
                    }
                    's' => {
                        spawns.push(Block(x as i8, y as i8));
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

        Ok(Map {
            wall_set,
            spawns,
            block_matrix,
        })
    }
}
