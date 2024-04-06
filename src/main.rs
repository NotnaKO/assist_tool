use crate::preparing::state::State;
use anyhow::Context;
use clap::{Parser, Subcommand};
use log::{debug, error, info, LevelFilter, trace};

mod preparing;

#[derive(Parser, Debug)]
#[command(author, version, about = "A helper tool for preparing C++ course")]
struct Args {
    /// Path to config of the author and the settings
    #[arg(short, long, default_value = "config.json")]
    config_path: String,

    /// Project directory path with structure from README.md
    #[arg(short, long, default_value = ".")]
    project_dir: String,

    /// Task to perform(from task list)
    #[arg(short, long)]
    task: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the review of the task
    Review,

    /// Add a new task to the project
    Add {
        /// Name of file with code to review
        #[arg(short, long)]
        code_file_name: String,
    },
}


fn main() -> anyhow::Result<()> {
    simple_logger::init().unwrap();

    let args = Args::parse();
    trace!("Args: {:?}", args);

    let mut state =
        State::load_state(args.config_path, args.project_dir).context("Can't load state")?;
    trace!("State load: {:?}", state);

    match args.command {
        Commands::Review => {
            trace!("Review command",);
            state
                .switch_to_task(&args.task)
                .context("Can't switch to task")?;
            trace!("State switched to the task: {:?}", state);
            state
                .check_environment(false)
                .context("Can't check environment")?;
            trace!("Environment checked");
        }
        Commands::Add { code_file_name } => {
            trace!("Add command");
            state
                .add_task(args.task, code_file_name)
                .context("Can't add task")?;
            state.dump_state()?
        }
    }
    Ok(())
}
