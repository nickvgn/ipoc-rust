use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub write_image: u8,
    pub repeat_count: u8,
}

pub fn get_config() -> Result<Config, envy::Error> {
    envy::from_env::<Config>()
}
