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
    blocks: HashSet<Block>,
    spawns: HashSet<Block>,
}

impl Map {
    pub fn from_file(path: &str) -> Result<Map, Error> {
        let file = File::open(path).map_err(|err| {
            Error::from(err, ConfigurationError).explain("configuration file not found")
        })?;
        let reader = BufReader::new(file);

        let mut blocks = HashSet::new();
        let mut spawns = HashSet::new();
        let mut y = 0;

        for line in reader.lines() {
            let row = line.map_err(|err| {
                Error::from(err, ConfigurationError).explain("could not parse line")
            })?;
            let chars: Vec<char> = row.chars().collect();

            for (x, ch) in chars.iter().enumerate() {
                match ch {
                    'x' => {
                        blocks.insert(Block(x as i8, y as i8));
                    }
                    's' => {
                        spawns.insert(Block(x as i8, y as i8));
                    }
                    _ => {}
                }
            }

            y += 1;
        }

        Ok(Map { blocks, spawns })
    }
}
