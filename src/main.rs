use renom::{
    cli::{self, get_help_text, Command},
    director,
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
        Some(comm) => match comm {
            Command::RenameProject => {
                println!("not yet implemented");
                // grab the options
                // generate context from it
                // validate options at the same time
                // create a changeset
                // execute changeset
                // revert if necessary
                // return non-zero if failure
            }
            Command::RenamePlugin => println!("not yet implemented"),
            Command::RenameTarget => println!("not yet implemented"),
            Command::RenameModule => println!("not yet implemented"),
            Command::Wizard => director::start_interactive_dialogue(),
        },
    }
}
