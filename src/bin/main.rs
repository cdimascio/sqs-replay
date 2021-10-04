use clap::{load_yaml, App};
use termion::{color};
use sqs::model::Message;
use sqs_replay::error::ReplayError;
use sqs_replay::replay;
use sqs_replay::replay::ISqsReplay;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), ReplayError> {
    let args = parse_args();
    let player = replay::SqsReplay::new(replay::SqsReplayOptions {
        region: args.region,
        source: args.source,
        dest: args.dest,
    })
    .await?;

    let v = args.verbose;
    let visitor = |m: &Message| {
        if v {
            match &m.body {
                Some(b) => println!("{}", b),
                None => println!("{{}}"),
            }
        } else {
            print!("{} {}", color::Fg(color::Green), ".");
            io::stdout().flush().unwrap();
        }
    };
    let r = player
        .replay(
            replay::ReplayOpts {
                max_messages: args.max_num_messages,
                selector: args.dedup_regex,
            },
            visitor,
        )
        .await;

    match r {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(r#"{}{{ "error": "{}" }}"#, color::Fg(color::Red), e.to_string());
            Ok(())
        }
    }
}

#[derive(Debug)]
struct CliArgs {
    source: String,
    dest: String,
    region: Option<String>,
    verbose: bool,
    max_num_messages: Option<i32>,
    dedup_regex: Option<String>,
}

fn parse_args() -> CliArgs {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let source = matches.value_of("source").map(|r| r.to_string()).unwrap();
    let dest = matches.value_of("dest").map(|r| r.to_string()).unwrap();
    let verbose = matches.is_present("verbose");
    let dedup_regex = matches
        .value_of("dedup-regex")
        .map(|r| r.to_string());
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
        verbose,
        max_num_messages,
        dedup_regex,
    }
}
