use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::{commands::LinkFormat, storage::get_default_storage_path};

#[derive(Debug, Deserialize)]
pub struct FiniConfig {
    #[serde(default = "default_list_link_format")]
    pub list_link_format: LinkFormat,
}

pub fn load_config() -> Result<FiniConfig, ConfigError> {
    let path = get_default_storage_path();
    let file_path = path.join("fini_config");
    let data = Config::builder()
        .add_source(File::from(file_path).required(false))
        .build()?;
    data.try_deserialize()
}

fn default_list_link_format() -> LinkFormat {
    LinkFormat::AdjacentColor
}
