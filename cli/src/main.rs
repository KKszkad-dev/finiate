use clap::Parser;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    Slot {
        #[command(subcommand)]
        slot_command: SlotCommands,
    },
    Mark {
        mark_log: Option<String>,
    },
    Putoff {
        putoff_log: Option<String>,
    },
    Terminate {
        terminate_log: Option<String>,
    },
}

#[derive(Parser, Debug)]
enum SlotCommands {
    // Define subcommands for Slot here
    Add {
        title: Option<String>,
        #[arg(long, short)]
        terminate_at: Option<String>,
    },
    Set {
        slot: Option<u8>,
    },
    Shelve {
        title: Option<String>,
    },
    History,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Slot { slot_command } => {
            match slot_command {
                SlotCommands::Add {
                    title,
                    terminate_at,
                } => {
                    println!(
                        "Adding slot with title: {:?} and terminate_at: {:?}",
                        title, terminate_at
                    );
                    // Implement the logic for adding a slot here
                }
                SlotCommands::Set { slot } => {
                    println!("Setting slot: {:?}", slot);
                    // Implement the logic for setting slots here
                }
                SlotCommands::History => {
                    println!("Showing slot history");
                    // Implement the logic for showing slot history here
                }
                SlotCommands::Shelve { title } => {
                    println!("Shelving current slot with title: {:?}", title);
                    // Implement the logic for shelving a slot here
                }
            }
        }
        Commands::Mark { mark_log } => {
            println!("Marking log content: {:?}", mark_log);
            // Implement the logic for marking log content here
        }
        Commands::Putoff { putoff_log } => {
            println!("Putting off log content: {:?}", putoff_log);
            // Implement the logic for putting off log content here
        }
        Commands::Terminate { terminate_log } => {
            println!("Terminating with log content: {:?}", terminate_log);
            // Implement the logic for terminating with log content here
        }
    }
}
