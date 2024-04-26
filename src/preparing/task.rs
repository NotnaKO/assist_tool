use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::path::Path;

use anyhow::{ensure, Context};
use log::trace;
use serde::{Deserialize, Serialize};

use crate::preparing::notes::{FileNotesStorage, Note};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Task {
    pub name: String,
    pub code_file_name: String,
    notes: FileNotesStorage<TaskNode, TaskNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskNode {
    text: String,
}

impl From<String> for TaskNode {
    fn from(text: String) -> Self {
        TaskNode { text }
    }
}

impl Display for TaskNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Note for TaskNode {
    fn text(&self) -> String {
        self.text.clone()
    }
}

impl Task {
    /// Create new task (in add task command)
    pub(super) fn new(
        project_dir: &Path,
        task_name: String,
        code_file_name: String,
    ) -> anyhow::Result<Self> {
        let task_dir = project_dir.join("tasks").join(&task_name);
        fs::create_dir_all(task_dir.as_path()).context("Can't create task directory")?;
        trace!("Task directory created {}", task_dir.display());

        let code_file_path = task_dir.join(&code_file_name);
        File::create_new(code_file_path).context("Can't create new file for code to task")?;
        trace!("File to code created");

        let notes = FileNotesStorage::new(
            project_dir
                .join("notes")
                .join(&task_name)
                .with_extension("txt"),
        )?;
        Ok(Task {
            name: task_name,
            code_file_name,
            notes,
        })
    }

    pub fn add_note(&mut self, text: String, optional: bool) {
        if optional {
            self.notes.add_optional_note(TaskNode::from(text));
        } else {
            self.notes.add_note(TaskNode::from(text));
        }
    }

    pub fn find_note(&self, num: usize, optional: bool) -> anyhow::Result<&TaskNode> {
        if optional {
            self.notes.find_optional_note(num)
        } else {
            self.notes.find_note(num)
        }
        .context("Note not found")
    }

    pub fn get_file(&self) -> anyhow::Result<File> {
        File::open(&self.code_file_name).context("Can't open file with code")
    }

    pub(super) fn check_environment(&self, project_dir: &Path) -> anyhow::Result<()> {
        let tasks_dir = project_dir.join("tasks").join(&self.name);
        trace!("Check task directory: {}", tasks_dir.display());
        ensure!(tasks_dir.exists(), "Task directory doesn't exist");
        ensure!(tasks_dir.is_dir(), "Task directory is not a directory");

        let notes_dir = project_dir.join("notes");
        trace!("Check notes directory: {}", notes_dir.display());
        ensure!(notes_dir.exists(), "Notes directory doesn't exist");
        ensure!(notes_dir.is_dir(), "Notes directory is not a directory");

        let task_code_file = tasks_dir
            .join(&self.name)
            .with_file_name(&self.code_file_name);
        trace!("Check task code file: {}", task_code_file.display());
        ensure!(task_code_file.exists(), "Task code file doesn't exist");
        ensure!(task_code_file.is_file(), "Task code file is not a file");

        let notes_file = notes_dir.join(&self.name).with_extension("txt");
        trace!("Check notes file: {}", notes_file.display());
        ensure!(notes_file.exists(), "Notes file doesn't exist");
        ensure!(notes_file.is_file(), "Notes file is not a file");

        Ok(())
    }
}
