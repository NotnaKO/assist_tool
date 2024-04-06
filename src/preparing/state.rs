use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, ensure, Context};
use log::trace;

use super::task::Task;

#[derive(Debug)]
pub struct State {
    author: Author,
    current_task: Option<usize>,
    tasks: Vec<Task>,
    project_dir: PathBuf,
    config_path: PathBuf,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    author_name: String,
    author_contacts: String,
    tasks: Vec<Task>,
}

impl State {
    /// Load state from the config file
    pub fn load_state(config_path: String, project_dir: String) -> anyhow::Result<Self> {
        let config_path = PathBuf::from(config_path);
        let config = serde_json::from_str::<Config>(
            &fs::read_to_string(&config_path).context("Failed to read config file")?,
        )?;
        Ok(State {
            author: Author::new(config.author_name, config.author_contacts),
            current_task: None,
            tasks: config.tasks,
            project_dir: project_dir.into(),
            config_path,
        })
    }

    /// Set the task to review
    pub fn switch_to_task(&mut self, task_name: &str) -> anyhow::Result<()> {
        let task = self
            .tasks
            .iter()
            .enumerate()
            .find(|(_, task)| task.name == task_name)
            .context("Task not found")?;
        self.current_task = task.0.into();
        Ok(())
    }

    /// Check the environment for the task
    pub fn check_environment(&self, create: bool) -> anyhow::Result<()> {
        if !self.project_dir.exists() {
            if create {
                fs::create_dir_all(&self.project_dir).context("Can't create project directory")?;
            } else {
                bail!("Project directory doesn't exist");
            }
        }
        ensure!(
            &self.project_dir.is_dir(),
            "Project directory is not a directory"
        );

        let task = self
            .current_task
            .as_ref()
            .context("Task is not set")
            .and_then(|task| self.tasks.get(*task).context("Task not found"))?;
        task.check_environment(&self.project_dir, create)?;

        Ok(())
    }

    /// Add new task
    pub fn add_task(&mut self, task_name: String, code_file_name: String) -> anyhow::Result<()> {
        trace!(
            "Start adding task {} with code_file_name {}",
            task_name,
            code_file_name
        );
        let task = Task::new(self.project_dir.as_path(), task_name, code_file_name)?;
        self.tasks.push(task);
        Ok(())
    }

    /// Save the state in the config
    pub fn dump_state(self) -> anyhow::Result<()> {
        let new_config = Config {
            author_name: self.author.name,
            author_contacts: self.author.contacts,
            tasks: self.tasks,
        };
        let value_to_write =
            serde_json::to_string_pretty(&new_config).context("Can't serialize state to json")?;
        fs::write(self.config_path, value_to_write).context("Can't write to the config path")
    }
}

#[derive(Debug)]
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
