use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use ratatui::widgets::BorderType;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub features: Features,
    pub view: View,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Features {
    pub data_path: String,
    pub default_folder: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct View {
    pub border_types: String,
}

impl Default for View {
    fn default() -> Self {
        Self {
            border_types: "rounded".to_string(),
        }
    }
}

impl View {
    #[allow(dead_code)]
    pub fn get_border_type(&self) -> BorderType {
        match self.border_types.to_lowercase().as_str() {
            #[allow(clippy::match_same_arms)]
            "rounded" => BorderType::Rounded,
            "thick" => BorderType::Thick,
            "double" => BorderType::Double,
            "plain" => BorderType::Plain,
            "quadrant" => BorderType::QuadrantOutside,
            _ => BorderType::Rounded,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let default_path = home.join("Documents/todo_files");

        Self {
            features: Features {
                data_path: default_path.to_string_lossy().to_string(),
                default_folder: "INBOX".to_string(),
            },
            view: View::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            let toml_content = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&toml_content)?;
            config.ensure_paths()?;
            Ok(config)
        } else {
            let default_config = Self::default();

            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            default_config.save()?;
            default_config.ensure_paths()?;

            Ok(default_config)
        }
    }

    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".config");
        path.push("todoCLI");
        path.push("config.toml");
        path
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path();
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn ensure_paths(&self) -> io::Result<()> {
        let clean_data_path = self.features.data_path.trim_end_matches('/').trim_end_matches('\\');

        let data_path = Path::new(clean_data_path);
        if !data_path.exists() {
            fs::create_dir_all(data_path)?;
        }

        let default_folder_path = format!("{}/{}", clean_data_path, self.features.default_folder);
        let default_folder = Path::new(&default_folder_path);
        if !default_folder.exists() {
            fs::create_dir_all(default_folder)?;
        }

        Ok(())
    }

    pub fn get_full_path(&self, folder: &str, filename: &str) -> String {
        let clean_data_path = self.features.data_path.trim_end_matches('/').trim_end_matches('\\');
        format!("{clean_data_path}/{folder}/{filename}")
    }

    pub fn get_folder_path(&self, folder: &str) -> String {
        let clean_data_path = self.features.data_path.trim_end_matches('/').trim_end_matches('\\');
        format!("{clean_data_path}/{folder}")
    }
}
