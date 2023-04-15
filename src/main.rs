use std::{fs, path::PathBuf};

use renom::{
    cli::{self, get_help_text, Command},
    director,
    engine::Engine,
    workflows::{self, changeset::generate_changeset, gather_context_from_input, validate_input},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let parsed_args = cli::parse_args(&args);
    if let Err((command, msg)) = parsed_args {
        let help_text = get_help_text(&command);
        println!("{help_text}\nerror: {msg}");
        return;
    }

    let (command, options) = parsed_args.unwrap();
    match command {
        None => {
            if options.contains_key("--help") {
                let help_text = get_help_text(&command);
                println!("{help_text}");
            } else if options.contains_key("--version") {
                let version = env!("CARGO_PKG_VERSION");
                println!("{version}");
            }
        }
        Some(command) => match command {
            Command::RenameProject => {
                // construct input from arguments
                let project_root = PathBuf::from(options["--project"].as_ref().unwrap());
                let new_name = options["--new-name"].as_ref().unwrap().clone();
                let input = workflows::rename_project::Input {
                    project_root,
                    new_name,
                };

                // validate input
                if let Err(e) = validate_input(&input) {
                    println!("invalid input: {e}");
                    return;
                }

                // gather context
                let context = match gather_context_from_input(&input) {
                    Ok(context) => context,
                    Err(e) => {
                        println!("failed to gather context: {e}");
                        return;
                    }
                };

                // build changeset
                let changeset = generate_changeset(&context);

                // execute (and revert on failure)
                let mut engine = Engine::new();
                let backup_dir = context.project_root.join(".renom").join("backup");
                fs::create_dir_all(&backup_dir).unwrap();
                if let Err(e) = engine.execute(changeset, backup_dir) {
                    println!("error while renaming project: {e}");
                    if let Err(e) = engine.revert() {
                        println!("error while reverting: {e}");
                    }
                }
            }
            Command::RenamePlugin => println!("not yet implemented"),
            Command::RenameTarget => println!("not yet implemented"),
            Command::RenameModule => println!("not yet implemented"),
            Command::Wizard => director::start_interactive_dialogue(),
        },
    }
}
