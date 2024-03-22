use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub monitor: String,
    pub wallpapers: String,
    pub debug: Option<bool>,
}
