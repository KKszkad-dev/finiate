use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// agenda serial number, default is 1
    #[arg(short, long, default_value_t = 1)]
    slot: u8,
    /// direct argument: log content
    #[arg(value_name = "LOG_CONTENT")]
    log_content: Option<String>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /***
     * add a new agenda to slots
     * left a start log
     * adjust the orders of agends in slots according to their terminate_at time.
     */
    Add {
        #[arg(value_name = "AGENDA_TITLE")]
        agenda_title: String,
        #[arg(value_name = "TERMINATE_AT", short, long)]
        terminate_at: String,
    },

    /***
     * show the status of the first `agenda_amount` agendas, default is 1, and the max is 5.
     */
    Status {
        #[arg(value_name = "AGENDA_ID", short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=5))]
        agenda_amount: u8,
        // TODO
        // make the agendas in and behind the slot all put off, left put-off logs in each agenda.
        // ripple: bool,
    },

    /***
     * put off the agenda located in the slot
     * left a put-off log
     * adjust the orders of agends in slots according to their terminate_at time.
     */
    PutOff {
        #[arg(value_name = "AGENDA_ID", short, long, default_value_t = 1)]
        slot: u8,
        #[arg(value_name = "PUT_OFF_CONTENT")]
        content: Option<String>,
    },

    /***
     * terminate the agenda located in the slot,
     * left a terminate log
     * adjust the orders of agends in slots according to their terminate_at time.
     */
    Terminate {
        #[arg(value_name = "AGENDA_ID", short, long, default_value_t = 1)]
        slot: u8,
        #[arg(value_name = "TERMINATE_CONTENT")]
        content: Option<String>,
    },

    /***
     * add a pending agenda (no terminate time yet)
     * (there shouldn't be multiple pending agendas using the same title)
     */
    Shelve {
        #[arg(value_name = "AGENDA_TITLE")]
        agenda_title: String,
    },
    /***
     * show the history of terminated and ongoing agendas and logs, sorted by many options.
     */
    History {
        // TODO
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Add {
                agenda_title,
                terminate_at,
            } => {
                println!(
                    "add agenda {}, terminate at: {}",
                    agenda_title, terminate_at
                );
            }
            Commands::PutOff { slot, content } => {
                if let Some(content) = content {
                    println!("put off agenda {}, content: {}", slot, content);
                } else {
                    println!("put off agenda {}", slot);
                }
            }
            Commands::Status { agenda_amount } => {
                println!("status of first {} agendas", agenda_amount);
            }
            Commands::Terminate { slot, content } => {
                if let Some(content) = content {
                    println!("terminate agenda {}, content: {}", slot, content);
                } else {
                    println!("terminate agenda {}", slot);
                }
            }
            Commands::Shelve { agenda_title } => {
                println!("shelve agenda {}", agenda_title);
            }
            Commands::History {} => {
                println!("show history of agendas and logs");
            }
        },
        None => {
            if let Some(log_content) = cli.log_content {
                println!("saved in agenda {}, log content: {}", cli.slot, log_content);
            } else {
                // deal with the none-command and none-log-content case
                println!(
                    "No command provided and log content is empty. Please provide a command or log content."
                );
            }
        }
    }
}
