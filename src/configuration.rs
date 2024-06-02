use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bucket_name: String,
}

pub fn get_config() -> Result<Config, envy::Error> {
    envy::from_env::<Config>()
}
