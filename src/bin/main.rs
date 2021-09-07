use clap::{load_yaml, App};
use sqs_replay_cli::sqsr::{self, Replayer};

#[tokio::main]
async fn main() -> Result<(), sqs::Error> {
    let args = parse_args();
    let player = sqsr::SqsReplayer::new(sqsr::SqsReplayerOpts {
        region: args.region,
    })
    .await?;
    let callback = |m: String| println!("processed {}", m);
    let r = player
        .replay(sqsr::ReplayOpts {
            source: args.source,
            dest: args.dest,
            max_num_messages: args.max_num_messages,
            callback,
        })
        .await;

    match r {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            Ok(())
        }
    }
}

struct CliArgs {
    source: String,
    dest: String,
    region: Option<String>,
    max_num_messages: Option<i32>,
}

fn parse_args() -> CliArgs {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let source = matches.value_of("source").map(|r| r.to_string()).unwrap();
    let dest = matches.value_of("dest").map(|r| r.to_string()).unwrap();
    let region = matches.value_of("region").map(|r| r.to_string());
    let r_max_message = matches.value_of("max-messages").map(|m| m.parse::<i32>());
    let max_num_messages = match r_max_message {
        Some(r) => match r {
            Ok(o) => Some(o),
            Err(error) => panic!("Can't parse --num-messages: {}", error),
        },
        None => None,
    };
    CliArgs {
        region,
        source,
        dest,
        max_num_messages,
    }
}
