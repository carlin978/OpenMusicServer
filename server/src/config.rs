use anyhow::{Context, bail};
use oms_types::Config;
use std::{env, path::PathBuf};

pub fn load_config() -> anyhow::Result<Config> {
    use config;

    let path = if cfg!(debug_assertions) {
        env::current_dir()?
    } else {
        let path = if cfg!(windows) {
            PathBuf::from(env::var("LOCALAPPDATA")?)
        } else {
            match env::var("XDG_CONFIG_HOME") {
                Ok(content) => PathBuf::from(content),
                Err(env::VarError::NotPresent) => PathBuf::from(env::var("HOME")?).join(".config"),
                Err(err) => bail!(err),
            }
        };

        path.join("oms").join("oms.conf")
    };

    Ok(config::Config::builder()
        .add_source(
            config::File::new(
                path.to_str()
                    .context("Non-Unicode paths aren't supported")?,
                config::FileFormat::Ini,
            )
            .required(false),
        )
        .add_source(config::Environment::with_prefix("OMS"))
        .build()
        .context("Failed to build")?
        .try_deserialize::<Config>()
        .context("Failed to deserialize into Config")?)
}

pub fn get_app_data_dir() -> anyhow::Result<PathBuf> {
    let mut path = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA")?)
    } else {
        match env::var("XDG_DATA_HOME") {
            Ok(content) => PathBuf::from(content),
            Err(env::VarError::NotPresent) => PathBuf::from(env::var("HOME")?)
                .join(".local")
                .join("share"),
            Err(err) => bail!(err),
        }
    };

    path.push("oms");

    Ok(path)
}
