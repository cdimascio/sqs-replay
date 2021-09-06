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
                .value_name("url")
                .about("The SQS source url")
                .takes_value(true),
        )
        .arg(
            Arg::new("dest")
                .short('d')
                .long("dest")
                .value_name("url")
                .about("The SQS desintation url")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .about("Sets an optional output file")
                .index(1),
        )
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    let source = match matches.value_of("source") {
        Some(o) => o,
        None => "",
    };

    let dest = match matches.value_of("dest") {
        Some(o) => o,
        None => "",
    };
    println!("{}", source);

    let player = sqsr::SqsReplayer();

    player
        .replay(sqsr::ReplayOpts {
            source: source.to_owned(),
            dest: dest.to_owned(),
        })
        .await?;

    println!("{:?}", matches);
 
    Ok(())
}

   // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // match matches.occurrences_of("debug") {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // // You can check for the existence of subcommands, and if found use their
    // // matches just as you would the top level app
    // if let Some(matches) = matches.subcommand_matches("test") {
    //     // "$ myapp test" was run
    //     if matches.is_present("list") {
    //         // "$ myapp test -l" was run
    //         println!("Printing testing lists...");
    //     } else {
    //         println!("Not printing testing lists...");
    //     }
    // }

