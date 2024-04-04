use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use anyhow::{ensure, Context};

#[derive(Debug, serde::Deserialize)]
pub struct Task {
    code_file_name: String,
    notes: NotesVec,
}

impl Task {
    pub fn new(project_dir: &Path, task_name: &str, code_file_name: &str) -> anyhow::Result<Self> {
        let task_dir = project_dir.join("tasks").join(task_name);
        fs::create_dir_all(&task_dir).context("Can't create task directory")?;

        let notes_dir = project_dir.join("notes");

        let notes = NotesVec::new(
            notes_dir
                .with_file_name(task_name)
                .with_extension("txt")
                .to_str()
                .context("Can't convert path")?,
        )?;

        Ok(Task {
            code_file_name: code_file_name.to_string(),
            notes,
        })
    }

    pub fn add_note(&mut self, note: &str, optional: bool) {
        self.notes.add_note(note, optional);
    }
}

#[derive(Debug, serde::Deserialize)]
struct NotesVec {
    /// Path to the display and store notes
    file_name: String,

    necessary_notes: Vec<String>,
    optional_notes: Vec<String>,
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
