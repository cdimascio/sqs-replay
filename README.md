# sqs-replay

## Usage

```shell
USAGE:
    main [FLAGS] [OPTIONS] --source <https://sqs-<region>.amazonaws.com/<account>/<quene-name>> --dest <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Output the contents of each SQS message replayed
    -V, --version    Print version information

OPTIONS:
    -d, --dest <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>
            The destination SQS url

    -m, --max-messages <1>
            The maximum number of messages to replay

    -r, --region <us-west-2>
            The AWS region

    -s, --source <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>
            The source SQS url
```

## Develop

```shell
cargo build
cargo run -- --source https://host/MyDLQueue --dest https://host/MyQueue -max-messages 2 --verbose
```

## Package

```shell
cargo build --release
```