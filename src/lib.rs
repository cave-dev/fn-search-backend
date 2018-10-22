

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

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

pub fn get_config() -> Result<Config, Box<Error>> {
    let mut f= File::open("./config.toml")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let x = toml::from_str(s.as_mut_str())?;
    Ok(x)
}

#[test]
fn test_load_config_file() {
    get_config().expect("error loading config file");
}
