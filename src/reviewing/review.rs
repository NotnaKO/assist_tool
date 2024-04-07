use std::fs::File;
use std::io::stdout;

use anyhow::Context;

use crate::preparing::context::{Author, ProjectContext};
use crate::preparing::notes::{FileNotesStorage, Note};
use crate::preparing::task::Task;

#[derive(Debug)]
pub(crate) struct Review {
    task: Task,
    author: Author,
    state: ReviewState,
    current_notes: FileNotesStorage<ReviewNote, ReviewNote>,
}

#[derive(Debug)]
enum ReviewState {
    Start,
    Review,
    Finish,
}

#[derive(Debug, Clone)]
struct ReviewNote {
    text: String,
    // todo: code_reference?
}

impl From<String> for ReviewNote {
    fn from(text: String) -> Self {
        Self { text }
    }
}

impl std::fmt::Display for ReviewNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Note for ReviewNote {
    fn text(&self) -> &str {
        &self.text
    }
}

enum ReviewAction {
    NewNote(ReviewNote, bool),
    AddNote(ReviewNote, bool),
    Show,
    Drop,
    Complete,
    Incorrect(String),
}

impl Review {
    pub(crate) fn new(mut context: ProjectContext) -> anyhow::Result<Self> {
        let task = context
            .tasks
            .swap_remove(context.current_task.context("Task is not set")?);
        let notes_file_name = context
            .project_dir
            .join("reviews")
            .join(&task.name)
            .with_extension("txt");
        File::create(&notes_file_name).context("Can't create notes file in reviews directory")?;
        Ok(Self {
            task,
            author: context.author,
            state: ReviewState::Start,
            current_notes: FileNotesStorage::new(notes_file_name)?,
        })
    }

    pub(crate) fn step(&mut self) -> anyhow::Result<()> {
        match self.state {
            ReviewState::Start => {
                println!("Let's start new review:");
                // todo: Last versions

                self.state = ReviewState::Review
            }
            ReviewState::Review => {
                let action = self
                    .ask_action()
                    .unwrap_or_else(|err| ReviewAction::Incorrect(err.to_string()));
                match action {
                    ReviewAction::NewNote(note, optional) => {
                        self.task.add_note(note.text, optional);
                        println!("Ok");
                    }
                    ReviewAction::AddNote(note, optional) => {
                        if optional {
                            self.current_notes.add_optional_note(note);
                        } else {
                            self.current_notes.add_note(note);
                        }
                        self.current_notes.save().context("Can't save notes")?;
                        println!("Ok");
                    }
                    ReviewAction::Show => {
                        self.show();
                    }
                    ReviewAction::Drop => {
                        self.current_notes.clear();
                        println!("Ok");
                    }
                    ReviewAction::Complete => {
                        self.finish_review()?;
                    }
                    ReviewAction::Incorrect(msg) => {
                        println!("{}", msg);
                    }
                }
            }
            ReviewState::Finish => {
                unreachable!("Finish state unreachable here")
            }
        }
        Ok(())
    }

    fn ask_action(&self) -> anyhow::Result<ReviewAction> {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Reading line fail")?;
        let mut tokens = input.split_whitespace();
        match tokens.next() {
            Some("new") | Some("n") => {
                let (optional, tokens) = Self::parse_optional(tokens)?;
                Ok(ReviewAction::NewNote(
                    ReviewNote {
                        text: tokens.collect::<Vec<_>>().join(" "),
                    },
                    optional,
                ))
            }
            Some("add") | Some("a") => {
                let (optional, tokens) = Self::parse_optional(tokens)?;
                Ok(ReviewAction::AddNote(
                    self.find_note(optional, tokens)?,
                    optional,
                ))
            }
            Some("show") | Some("s") => Ok(ReviewAction::Show),
            Some("complete") | Some("c") => Ok(ReviewAction::Complete),
            Some("drop") | Some("d") => Ok(ReviewAction::Drop),
            _ => Ok(ReviewAction::Incorrect("Unknown action".to_string())),
        }
    }

    fn parse_optional<'a>(
        tokens: impl Iterator<Item = &'a str>,
    ) -> anyhow::Result<(bool, impl Iterator<Item = &'a str>)> {
        let mut tokens = tokens.peekable();
        let optional = matches!(*tokens.peek().context("No text in note")?, "optional" | "o");
        if optional {
            tokens.next();
        }
        Ok((optional, tokens))
    }

    fn find_note<'a>(
        &self,
        optional: bool,
        mut tokens: impl Iterator<Item = &'a str>,
    ) -> anyhow::Result<ReviewNote> {
        let num = tokens
            .next()
            .context("No number in note")?
            .parse()
            .context("Incorrect number of note")?;
        Ok(ReviewNote {
            text: self.task.find_note(num, optional)?.to_string(),
        })
    }

    fn show(&self) {
        println!("{}", self.author);
        let mut writer = std::io::BufWriter::new(stdout());
        self.current_notes.save_with_writer(&mut writer).unwrap()
    }

    fn finish_review(&mut self) -> anyhow::Result<()> {
        self.state = ReviewState::Finish;
        println!("Review finished");
        Ok(())
    }

    pub(crate) fn is_finished(&self) -> bool {
        matches!(self.state, ReviewState::Finish)
    }
}
