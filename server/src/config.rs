use oms_types::Config;
use std::{env, fs, path::PathBuf};

pub fn load_config() -> anyhow::Result<Config> {
    let mut path = PathBuf::from(env::var(if cfg!(windows) {
        "LOCALAPPDATA"
    } else {
        "HOME"
    })?);

    path.push("oms");
    path.push("oms.conf");

    let file = fs::read_to_string(path)?;

    Ok(serde_ini::from_str(&file)?)
}
