use serde::Deserialize;

pub mod tasker;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    pub port: u16,
}
