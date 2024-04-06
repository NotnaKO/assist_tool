use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, ensure, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub name: String,
    code_file_name: String,
    notes: NotesVec,
}

impl Task {
    pub fn new(
        project_dir: &Path,
        task_name: String,
        code_file_name: String,
    ) -> anyhow::Result<Self> {
        let task_dir = project_dir.join("tasks").join(&task_name);
        fs::create_dir_all(task_dir).context("Can't create task directory")?;

        let notes_dir = project_dir.join("notes");

        let notes = NotesVec::new(
            notes_dir
                .with_file_name(&task_name)
                .with_extension("txt")
                .to_str()
                .context("Can't convert path")?,
        )?;

        Ok(Task {
            name: task_name,
            code_file_name,
            notes,
        })
    }

    pub fn add_note(&mut self, note: &str, optional: bool) {
        self.notes.add_note(note, optional);
    }

    pub fn check_environment(&self, project_dir: &Path, create: bool) -> anyhow::Result<()> {
        let tasks_dir = project_dir.join("tasks").join(&self.name);
        if !tasks_dir.exists() {
            if create {
                fs::create_dir_all(&tasks_dir).context("Can't create task directory")?;
            } else {
                bail!("Task directory doesn't exist");
            }
        }
        ensure!(tasks_dir.is_dir(), "Task directory is not a directory");

        let notes_dir = project_dir.join("notes");
        if !notes_dir.exists() {
            if create {
                fs::create_dir_all(&notes_dir).context("Can't create notes directory")?;
            } else {
                bail!("Notes directory doesn't exist");
            }
        }
        ensure!(notes_dir.is_dir(), "Notes directory is not a directory");

        let task_code_file = tasks_dir.with_file_name(&self.code_file_name);
        if !task_code_file.exists() {
            if create {
                File::create(&task_code_file).context("Can't create task code file")?;
            } else {
                bail!("Task code file doesn't exist");
            }
        }
        ensure!(task_code_file.is_file(), "Task code file is not a file");

        let notes_file = notes_dir.with_file_name(&self.name).with_extension("txt");
        if !notes_file.exists() {
            if create {
                File::create(&notes_file).context("Can't create notes file")?;
            } else {
                bail!("Notes file doesn't exist");
            }
        }
        ensure!(notes_file.is_file(), "Notes file is not a file");

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "&str", into = "String")]
struct NotesVec {
    /// Path to the display and store notes
    file_name: String,

    necessary_notes: Vec<String>,

    optional_notes: Vec<String>,
}

impl From<NotesVec> for String {
    fn from(value: NotesVec) -> Self {
        value.file_name
    }
}

impl NotesVec {
    fn new(file_name: &str) -> anyhow::Result<Self> {
        file_name.try_into()
    }

    fn parse_line(line: &str) -> anyhow::Result<(usize, &str)> {
        let index = line.find(')').context("Incorrect line")?;
        let num_str = &line[..index];
        let num = num_str.parse::<usize>().context("Incorrect number")?;
        Ok((num, &line[index + 1..]))
    }

    fn add_note(&mut self, note: &str, optional: bool) {
        if optional {
            self.optional_notes.push(note.to_string());
        } else {
            self.necessary_notes.push(note.to_string());
        }
        self.save().expect("Can't save notes");
    }

    fn save(&self) -> anyhow::Result<()> {
        let file = File::create(&self.file_name).context("Can't create file")?;

        let mut writer = std::io::BufWriter::new(file);

        writeln!(writer, "Necessary:")?;
        for (num, note) in self.necessary_notes.iter().enumerate() {
            writeln!(writer, "{}) {}", num, note)?;
        }
        if !self.optional_notes.is_empty() {
            writeln!(writer, "Optional:")?;
            for (num, note) in self.optional_notes.iter().enumerate() {
                writeln!(writer, "{}) {}", num, note)?;
            }
        }
        writer.flush()?;
        Ok(())
    }
}

impl TryFrom<&str> for NotesVec {
    type Error = anyhow::Error;

    fn try_from(file_name: &str) -> Result<Self, Self::Error> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_name)
            .expect("Can't open file for notes");

        let mut necessary_notes = Vec::new();
        let mut optional_notes = Vec::new();

        let mut lines = std::io::BufReader::new(file).lines();
        ensure!(
            lines.next().context("Empty file")?? == "Necessary:",
            "First line should be 'Necessary:'"
        );

        let mut optional = false;
        for (num, res) in (&mut lines).enumerate() {
            let line = res?;
            if line == "Optional:" {
                optional = true;
                break;
            }
            let (parsed_num, line) = NotesVec::parse_line(&line)?;
            ensure!(parsed_num == num, "Incorrect number of line");

            necessary_notes.push(line.to_string());
        }
        if !optional {
            return Ok(NotesVec {
                file_name: file_name.to_string(),
                necessary_notes,
                optional_notes,
            });
        }

        for (num, res) in lines.enumerate() {
            let line = res?;
            let (parsed_num, line) = NotesVec::parse_line(&line)?;
            ensure!(parsed_num == num, "Incorrect number of line");

            optional_notes.push(line.to_string());
        }

        Ok(NotesVec {
            file_name: file_name.to_string(),
            necessary_notes,
            optional_notes,
        })
    }
}
