use anyhow::{ensure, Context};
use log::trace;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, Write};
use std::path::PathBuf;

pub trait Note: Debug + Clone + From<String> {
    fn text(&self) -> String;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct FileNotesStorage<N: Note, O: Note> {
    /// Path to the display and store notes
    file_name: PathBuf,

    necessary_notes: Vec<N>,

    optional_notes: Vec<O>,
}

impl<N: Note, O: Note> From<FileNotesStorage<N, O>> for String {
    fn from(value: FileNotesStorage<N, O>) -> Self {
        value.file_name.to_string_lossy().into()
    }
}

impl<N: Note, O: Note> FileNotesStorage<N, O> {
    pub(crate) fn new(file_name: PathBuf) -> anyhow::Result<Self> {
        file_name.try_into()
    }

    fn parse_line(line: &str) -> anyhow::Result<(usize, &str)> {
        let index = line.find(')').context("Incorrect line")?;
        let num_str = &line[..index];
        let num = num_str.parse::<usize>().context("Incorrect number")?;
        Ok((num, &line[index + 1..]))
    }

    pub(crate) fn add_note(&mut self, note: N) {
        self.necessary_notes.push(note);
        self.save().expect("Can't save notes");
    }

    pub(crate) fn add_optional_note(&mut self, note: O) {
        self.optional_notes.push(note);
        self.save().expect("Can't save notes");
    }

    pub(crate) fn find_note(&self, num: usize) -> anyhow::Result<&N> {
        self.necessary_notes.get(num).context("Note not found")
    }

    pub(crate) fn find_optional_note(&self, num: usize) -> anyhow::Result<&O> {
        self.optional_notes.get(num).context("Note not found")
    }

    pub fn clear(&mut self) {
        self.necessary_notes.clear();
        self.optional_notes.clear();
        self.save().expect("Can't save notes");
    }

    pub(crate) fn save(&self) -> anyhow::Result<()> {
        let file = File::create(&self.file_name).context("Can't create file")?;

        let mut writer = std::io::BufWriter::new(file);

        self.save_with_writer(&mut writer)
    }

    pub(crate) fn save_with_writer(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        if !self.necessary_notes.is_empty() {
            writeln!(writer, "Necessary:")?;
            for (num, note) in self.necessary_notes.iter().enumerate() {
                writeln!(writer, "{}) {}", num, note.text())?;
            }
        }
        if !self.optional_notes.is_empty() {
            writeln!(writer, "Optional:")?;
            for (num, note) in self.optional_notes.iter().enumerate() {
                writeln!(writer, "{}) {}", num, note.text())?;
            }
        }
        writer.flush()?;
        Ok(())
    }
}

impl<N: Note, O: Note> TryFrom<String> for FileNotesStorage<N, O> {
    type Error = anyhow::Error;

    fn try_from(file_name: String) -> Result<Self, Self::Error> {
        let file_name = PathBuf::from(file_name);
        FileNotesStorage::try_from(file_name)
    }
}

impl<N: Note, O: Note> TryFrom<PathBuf> for FileNotesStorage<N, O> {
    type Error = anyhow::Error;

    fn try_from(file_name: PathBuf) -> Result<Self, Self::Error> {
        trace!("Try to open file: {:?}", &file_name);
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&file_name)
            .context("Can't open file for notes")?;
        trace!("File opened: {:?}", file);

        let mut necessary_notes = Vec::new();
        let mut optional_notes = Vec::new();

        let mut lines = std::io::BufReader::new(file).lines();
        match lines.next() {
            None => {
                trace!("Empty file");
                return Ok(FileNotesStorage {
                    file_name,
                    necessary_notes,
                    optional_notes,
                });
            }
            Some(first_line) => {
                ensure!(
                    first_line.context("First line read problem")? == "Necessary:",
                    "First line should be 'Necessary:'"
                );
            }
        };
        trace!("First line checked");

        let mut optional = false;
        for (num, res) in (&mut lines).enumerate() {
            let line = res?;
            if line == "Optional:" {
                optional = true;
                break;
            }
            let (parsed_num, line) = Self::parse_line(&line)?;
            ensure!(parsed_num == num, "Incorrect number of line");

            necessary_notes.push(line.to_string().into());
        }
        trace!("Not optional notes read");

        if !optional {
            return Ok(FileNotesStorage {
                file_name,
                necessary_notes,
                optional_notes,
            });
        }

        for (num, res) in lines.enumerate() {
            let line = res?;
            let (parsed_num, line) = Self::parse_line(&line)?;
            ensure!(parsed_num == num, "Incorrect number of line");

            optional_notes.push(line.to_string().into());
        }
        trace!("Optional notes read");

        Ok(FileNotesStorage {
            file_name,
            necessary_notes,
            optional_notes,
        })
    }
}
