use anyhow::Context;
use clap::{Parser, Subcommand};
use log::{info, trace};

use crate::preparing::context::ProjectContext;

mod preparing;
mod reviewing;

#[derive(Parser, Debug)]
#[command(author, version, about = "A helper tool for assisting C++ course")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    /// Start the reviewing of the task
    Review {
        /// Path to config of the author and the settings
        #[arg(long, default_value = "config.json")]
        config_path: String,

        /// Project directory path with structure from README.md
        #[arg(short, long, default_value = ".")]
        project_dir: String,

        /// Task to perform(from task list)
        #[arg(short, long)]
        task: String,
    },

    /// Add a new task to the project
    Add {
        /// Path to config of the author and the settings
        #[arg(long, default_value = "config.json")]
        config_path: String,

        /// Project directory path with structure from README.md
        #[arg(short, long, default_value = ".")]
        project_dir: String,

        /// Task to perform(from task list)
        #[arg(short, long)]
        task: String,

        /// Name of file with code to reviewing
        #[arg(short, long)]
        code_file_name: String,

        /// File name to show if you want to use file show method
        #[arg(short, long)]
        show_file_name: Option<String>,
    },

    /// Init project directory at current directory with config file at config.json
    Init {
        /// Author name and surname
        #[arg(short, long)]
        author: String,

        /// Contacts of the author (Telegram for example)
        #[arg(short, long)]
        contacts: String,
    },
}

fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Warn).unwrap();

    let args = Args::parse();
    trace!("Args: {:?}", args);

    match args.command {
        Commands::Init { author, contacts } => return ProjectContext::init_state(author, contacts),
        Commands::Review {
            task,
            config_path,
            project_dir,
        } => {
            let mut context = ProjectContext::load_state(config_path, project_dir)
                .context("Can't load context")?;
            info!("Context load: {:?}", context);
            info!("Review command",);
            context
                .switch_to_task(&task)
                .context("Can't switch to task")?;
            trace!("State switched to the task {}", task);
            context.check_task(&task).context("Check task fail")?;
            trace!("Task checked");
            println!("Start review with task: {}", task);
            start_review(context)?
        }
        Commands::Add {
            config_path,
            project_dir,
            task,
            code_file_name,
            show_file_name,
        } => {
            let mut context = ProjectContext::load_state(config_path, project_dir)
                .context("Can't load context")?;
            info!("Context load: {:?}", context);
            info!("Add command");
            let show_method = match show_file_name {
                Some(file_name) => {
                    let file_name = context
                        .project_dir
                        .join("tasks")
                        .join(task.as_str())
                        .join(file_name);
                    preparing::task::ShowMethod::File { file_name }
                }
                None => preparing::task::ShowMethod::Console,
            };
            context
                .add_task(task, code_file_name, show_method)
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
