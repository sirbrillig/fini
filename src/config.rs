use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::{commands::LinkFormat, storage::get_default_data_dir};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PrompterType {
    #[default]
    Inquire,
    Vim,
}

#[derive(Debug, Deserialize)]
pub struct FiniConfig {
    #[serde(default = "default_list_link_format")]
    pub list_link_format: LinkFormat,
    #[serde(default)]
    pub prompter: PrompterType,
}

pub fn load_config() -> Result<FiniConfig, ConfigError> {
    let path = get_default_data_dir();
    let file_path = path.join("fini_config");
    let data = Config::builder()
        .add_source(File::with_name(&file_path.to_string_lossy()).required(false))
        .build()?;
    data.try_deserialize()
}

fn default_list_link_format() -> LinkFormat {
    LinkFormat::Newline
}
