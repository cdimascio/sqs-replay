<p align="center">
    <img src="https://img.shields.io/badge/install-homebrew-yellow"/>
    <img src="https://img.shields.io/badge/license-MIT-blue.svg"/>
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/cdimascio/sqs-replay/main/assets/sqs-replay-logo.png"/>
</p>


## Install

MacOS

```shell
brew install cdimascio/tap/sqs_replay
```

## Usage

```shell
sqs-replay \
  --source https://host/MyDLQueue \
  --dest https://host/MyQueue \
  --max-messages 10 \
  --verbose
```

Note: exclude `max-messages` to replay all

## Options

```shell
USAGE:
    sqs-replay [FLAGS] [OPTIONS] --source <https://sqs-<region>.amazonaws.com/<account>/<quene-name>> --dest <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>

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


## License 
[MIT](LICENSE)
