use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context};
use log::trace;

use super::task::Task;

#[derive(Debug)]
pub struct ProjectContext {
    pub(crate) author: Author,
    pub(crate) current_task: Option<usize>,
    pub(crate) tasks: Vec<Task>,
    pub(crate) project_dir: PathBuf,
    config_path: PathBuf,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    author_name: String,
    author_contacts: String,
    tasks: Vec<Task>,
}

impl ProjectContext {
    /// Load state from the config file
    pub fn load_state(config_path: String, project_dir: String) -> anyhow::Result<Self> {
        let project_dir = PathBuf::from(project_dir);
        Self::check_environment(&project_dir)?;
        trace!("Project directories checked");
        
        let config_path = PathBuf::from(config_path);
        trace!("Load state from {}", config_path.display());
        let config = serde_json::from_str::<Config>(
            &fs::read_to_string(&config_path).context("Failed to read config file")?,
        )?;
        trace!("Config loaded: {:?}", config);
        Ok(ProjectContext {
            author: Author::new(config.author_name, config.author_contacts),
            current_task: None,
            tasks: config.tasks,
            project_dir: project_dir.into(),
            config_path,
        })
    }

    /// Set the task to reviewing
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
    pub fn check_environment(project_dir: &Path) -> anyhow::Result<()> {
        ensure!(project_dir.exists(), "Project directory doesn't exist");

        ensure!(
            &project_dir.is_dir(),
            "Project directory is not a directory"
        );
        trace!("Project checked");

        let review_dir = project_dir.join("reviews");
        fs::create_dir_all(review_dir.as_path()).context("Can't create review directory")?;
        trace!("Review directory created {}", review_dir.display());

        let tasks_dir = project_dir.join("tasks");
        fs::create_dir_all(tasks_dir.as_path()).context("Can't create task directory")?;
        trace!("Task directory created {}", tasks_dir.display());

        let notes_dir = project_dir.join("notes");
        fs::create_dir_all(notes_dir.as_path()).context("Can't create notes directory")?;
        trace!("Notes directory created {}", notes_dir.display());

        Ok(())
    }

    pub(crate) fn check_task(&self, task_name: &str) -> anyhow::Result<()> {
        let task = self
            .tasks
            .iter()
            .find(|task| task.name == task_name)
            .context("Task not found")?;
        task.check_environment(&self.project_dir)
    }

    /// Add new task
    pub(crate) fn add_task(
        &mut self,
        task_name: String,
        code_file_name: String,
    ) -> anyhow::Result<()> {
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
pub(crate) struct Author {
    pub name: String,
    pub contacts: String,
}

impl Author {
    pub fn new(name: String, contacts: String) -> Self {
        Self { name, contacts }
    }
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Author: {}(tg: {})", self.name, self.contacts)
    }
}
