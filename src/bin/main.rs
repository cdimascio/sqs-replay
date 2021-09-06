use clap::{App, Arg};
use sqs_replay_cli::sqsr::{self, Replayer};

#[tokio::main]
async fn main() -> Result<(), sqs::Error> {
    tracing_subscriber::fmt::init();
    let matches = App::new("sqs-replay-cli")
        .version("1.0")
        .author("Carmine DiMascio. <cdimascio@gmail.com>")
        .about("Replay SQS messages")
        .license("MIT")
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .required(true)
                .value_name("url")
                .about("The SQS source url")
                .takes_value(true),
        )
        .arg(
            Arg::new("dest")
                .short('d')
                .long("dest")
                .required(true)
                .value_name("url")
                .about("The SQS desintation url")
                .takes_value(true),
        )
        .arg(
            Arg::new("region")
                .short('r')
                .long("region")
                .value_name("region")
                .about("the region")
                .takes_value(true),
        )
        .arg(
            Arg::new("num-messages")
                .short('n')
                .long("num-messages")
                .value_name("url")
                .about("the maximum number of messages to replay")
                .takes_value(true),
        )
        .get_matches();

    let source = matches.value_of("source").map(|r| r.to_owned()).unwrap();
    let dest = matches.value_of("dest").map(|r| r.to_owned()).unwrap();
    let region = matches.value_of("region").map(|r| r.to_owned());

    let r_max_message = matches
        .value_of("max_number_messages")
        .unwrap_or("1")
        .parse::<i32>();
    let max_num_messages = match r_max_message {
        Ok(o) => Some(o),
        Err(error) => panic!("Can't parse --num-messages: {}", error),
    };


    let player = sqsr::SqsReplayer::new().await?;
    player
        .replay(sqsr::ReplayOpts {
            region,
            source,
            dest,
            max_num_messages,
        })
        .await?;

    println!("{:?}", matches);

    Ok(())
}