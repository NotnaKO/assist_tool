use anyhow::Context;
use clap::{Parser, Subcommand};
use log::{info, trace};

use crate::preparing::context::ProjectContext;

mod preparing;
mod reviewing;

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
    /// Start the reviewing of the task
    Review,

    /// Add a new task to the project
    Add {
        /// Name of file with code to reviewing
        #[arg(short, long)]
        code_file_name: String,
    },
}

fn main() -> anyhow::Result<()> {
    simple_logger::init().unwrap();

    let args = Args::parse();
    trace!("Args: {:?}", args);
    

    let mut context = ProjectContext::load_state(args.config_path, args.project_dir)
        .context("Can't load context")?;
    info!("Context load: {:?}", context);

    match args.command {
        Commands::Review => {
            info!("Review command",);
            context
                .switch_to_task(&args.task)
                .context("Can't switch to task")?;
            trace!("State switched to the task {}", args.task);
            context.check_task(&args.task).context("Check task fail")?;
            trace!("Task checked");
            println!("Start review with task: {}", args.task);
            start_review(context)?
        }
        Commands::Add { code_file_name } => {
            info!("Add command");
            context
                .add_task(args.task, code_file_name)
                .context("Can't add task")?;
            context.dump_state()?;
            println!("Successfully add");
        }
    }
    Ok(())
}

pub(crate) fn start_review(context: ProjectContext) -> anyhow::Result<()> {
    let mut review = reviewing::review::Review::new(context)?;
    while !review.is_finished() {
        review.step()?;
    }

    Ok(())
}
