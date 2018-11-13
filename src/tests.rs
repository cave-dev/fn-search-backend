use crate::get_config;

static RELATIVE_CFG_FILE: &'static str = "./config.toml";

#[test]
fn test_load_config_file() {
    get_config(RELATIVE_CFG_FILE).expect("error loading config file");
}
