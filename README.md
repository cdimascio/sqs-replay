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
sqs-replay --source https://host/MyDLQueue --dest https://host/MyQueue -max-messages 2 --verbose
```

## Options

```shell
USAGE:
    sqs-replay [FLAGS] [OPTIONS] --source <https://sqs-<region>.amazonaws.com/<account>/<quene-name>> --dest <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Outputs the contents of each SQS message replayed
    -V, --version    Print version information

OPTIONS:
    -d, --dest <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>
            The destination SQS url

    -j, --selector-json <userId":"(.*?)">
            Dedup selector. A regex that MUST contain a single capture group which captures the
            'deduplication key'.

    -m, --max-messages <1>
            The maximum number of messages to replay

    -r, --region <us-west-2>
            The AWS region

    -s, --source <https://sqs-<region>.amazonaws.com/<account>/<quene-name>>
            The source SQS url

```

## Dedup Messages

sqs-replay will deduplicate messages. Provide a regex with a single capture group the selects a key to dedup by

```shell
sqs-replay \ 
  --source https://host/MyDLQueue \ 
  --dest https://host/MyQueue \ 
  --max-messages 200 
  --verbose
  --selector-regex 'id":"(.*?)"'
```

The `selector-regex`, `userId":"(.*?)"` matches the value of `id` in a json structure. 

For example, if the following message were in the queue:

```shell
{
  "id": "12345",
  "name": "eliana"
}
```

```shell
{
  "id": "45678",
  "name": "luca"
}
```

```shell
{
  "id": "12345",
  "name": "eliana"
}
```

The `--selector-regex` will replay message 1 and 2, and delete 3.

## License 
MIT
