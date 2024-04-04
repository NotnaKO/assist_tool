use crate::assisting::task::Task;
use anyhow::Context;
use std::fmt::Display;
use std::fs;

pub struct State {
    author: Author,
    current_task: Option<usize>,
    tasks: Vec<Task>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    author: String,
    contacts: String,
    tasks: Vec<Task>,
}

impl From<Config> for State {
    fn from(config: Config) -> Self {
        Self {
            author: Author::new(config.author, config.contacts),
            current_task: None,
            tasks: config.tasks,
        }
    }
}

impl State {
    pub fn load_state(config_path: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str::<Config>(
            &fs::read_to_string(config_path).context("Failed to read config file")?,
        )?;
        Ok(config.into())
    }
}

struct Author {
    name: String,
    contacts: String,
}

impl Author {
    pub fn new(name: String, contacts: String) -> Self {
        Self { name, contacts }
    }
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Author {}(tg: {})", self.name, self.contacts)
    }
}
