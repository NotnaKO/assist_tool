use anyhow::Context;
use std::fs::File;
use std::io::{BufRead, BufReader};

use itertools::Itertools;
use log::trace;

use crate::preparing::notes::Note;

use crate::separator;
use const_format::{concatcp, str_repeat};


#[derive(Debug, Clone)]
pub(super) struct ReviewNote {
    text: String,
    references: Vec<String>,
}

pub(super) enum NoteType {
    Necessary,
    Optional,
    NecessaryWithReference((usize, usize)),
    OptionalWithReference((usize, usize)),
}

pub(super) fn parse_type<'a>(
    tokens: impl Iterator<Item = &'a str>,
) -> anyhow::Result<(NoteType, impl Iterator<Item = &'a str>)> {
    let mut tokens = tokens.peekable();
    let optional = matches!(*tokens.peek().context("No text in note")?, "optional" | "o");
    if optional {
        tokens.next();
    }
    let reference = matches!(
        *tokens.peek().context("No text in note")?,
        "reference" | "r"
    );
    if reference {
        tokens.next();
        let first = tokens
            .next()
            .context("No first number in reference")?
            .parse()
            .context("Incorrect first number in reference")?;
        let second = tokens
            .next()
            .context("No second number in reference")?
            .parse()
            .context("Incorrect second number in reference")?;
        if optional {
            Ok((NoteType::OptionalWithReference((first, second)), tokens))
        } else {
            Ok((NoteType::NecessaryWithReference((first, second)), tokens))
        }
    } else if optional {
        Ok((NoteType::Optional, tokens))
    } else {
        Ok((NoteType::Necessary, tokens))
    }
}

impl ReviewNote {
    pub fn new(text: String) -> Self {
        Self {
            text,
            references: Vec::new(),
        }
    }

    const NOTE_SEPARATOR: &'static str = separator!("-", 50);

    pub fn add_code_reference(&mut self, file: File, row_numbers: (usize, usize)) {
        let text = BufReader::new(file)
            .lines()
            .enumerate()
            .skip(row_numbers.0 - 1)
            .take(row_numbers.1 - row_numbers.0 + 1)
            .map(|(i, line)| format!("{:4}: {}", i + 1, line.unwrap()))
            .join("\n");
        trace!(
            "Reference added by rows: {}, {}",
            row_numbers.0,
            row_numbers.1
        );
        self.references.push(text);
    }

    pub fn text_to_storage(self) -> String {
        self.text
    }
}

impl From<String> for ReviewNote {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl Note for ReviewNote {
    fn text(&self) -> String {
        let mut text = self.text.clone();
        let references = self.references.join(Self::NOTE_SEPARATOR);
        if !references.is_empty() {
            text.push_str(Self::NOTE_SEPARATOR);
            text.push_str(&references);
            text.push_str(Self::NOTE_SEPARATOR);
        }
        text
    }
}
