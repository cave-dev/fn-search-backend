

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

#[derive(Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u32,
    pub db: String,
    pub user: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub db: DbConfig,
}

pub fn get_config(f: &str) -> Result<Config, Box<Error>> {
    let mut f= File::open(f)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let x = toml::from_str(s.as_mut_str())?;
    Ok(x)
}
