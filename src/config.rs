use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramEntry {
    pub name: String,
    pub display_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub description: Option<String>,
    pub run_with_sudo: bool,
    pub show_output: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub programs: HashMap<String, ProgramEntry>,
}

impl Default for Config {
    fn default() -> Self {
        let mut programs = HashMap::new();
        
        // Add some default programs
        programs.insert(
            "htop".to_string(),
            ProgramEntry {
                name: "htop".to_string(),
                display_name: "System Monitor".to_string(),
                command: "htop".to_string(),
                args: vec![],
                description: Some("System resource monitor".to_string()),
                run_with_sudo: false,
                show_output: false,
            },
        );
        
        programs.insert(
            "vim".to_string(),
            ProgramEntry {
                name: "vim".to_string(),
                display_name: "Text Editor".to_string(),
                command: "vim".to_string(),
                args: vec![],
                description: Some("Vim text editor".to_string()),
                run_with_sudo: false,
                show_output: false,
            },
        );

        Self { programs }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
            .join("rdash")
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = Self::config_file();
        
        if config_file.exists() {
            let content = fs::read_to_string(&config_file)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir)?;
        
        let config_file = Self::config_file();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        
        Ok(())
    }

    pub fn add_program(&mut self, entry: ProgramEntry) {
        self.programs.insert(entry.name.clone(), entry);
    }

    pub fn remove_program(&mut self, name: &str) -> bool {
        self.programs.remove(name).is_some()
    }

    pub fn get_programs(&self) -> Vec<&ProgramEntry> {
        let mut programs: Vec<_> = self.programs.values().collect();
        programs.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        programs
    }
}
