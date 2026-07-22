use std::path::PathBuf;

use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::{commands::LinkFormat, storage::get_default_data_dir};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PrompterType {
    #[default]
    Inquire,
    Vim,
}

#[derive(Debug, Deserialize)]
pub struct FiniConfig {
    #[serde(default = "default_list_link_format")]
    pub list_link_format: LinkFormat,
    #[serde(default = "default_copy_link_format")]
    pub copy_link_format: LinkFormat,
    #[serde(default)]
    pub prompter: PrompterType,
    /// Override where boards are stored. Supports ~ for home directory.
    /// Defaults to the platform data directory if not set.
    #[serde(default)]
    pub data_dir: Option<String>,
}

pub fn get_config_file_path_base() -> PathBuf {
    let path = get_default_data_dir();
    path.join("fini_config")
}

pub fn get_current_config_file_path() -> PathBuf {
    // This should match the features set for the config crate in Config.yaml. If that ever
    // changes, make sure to update this too.
    let formats = ["toml", "json", "yaml"];
    let file_path = get_config_file_path_base();
    formats
        .iter()
        .map(|e| file_path.with_extension(e))
        .find(|path| path.exists())
        .unwrap_or_else(|| file_path.with_extension("toml"))
}

pub fn load_config() -> Result<FiniConfig, ConfigError> {
    let file_path = get_config_file_path_base();
    let data = Config::builder()
        .add_source(File::with_name(&file_path.to_string_lossy()).required(false))
        .build()?;
    data.try_deserialize()
}

fn default_list_link_format() -> LinkFormat {
    LinkFormat::Newline
}

fn default_copy_link_format() -> LinkFormat {
    LinkFormat::Adjacent
}
