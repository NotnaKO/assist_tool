use clap::Parser;
use crate::assisting::state::State;

mod assisting;

#[derive(Parser, Debug)]
#[command(version, about = "A helper tool for assisting C++ course")]
struct CmdArgs {
    /// Path to config of the author and the settings
    #[arg(short, long, default_value = "config.json")]
    config_path: String,
    
    /// Project directory path with structure from README.md
    #[arg(short, long, default_value = ".")]
    project_dir: String,
    
    /// Task to perform(from task list)
    #[arg(short, long)]
    task: String,
}


fn main() {
    let args = CmdArgs::parse();
    let mut state = State::load_state(&args.config_path);
}
