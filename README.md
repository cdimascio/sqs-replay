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
    sqs-replay [FLAGS] [OPTIONS] --source <https://sqs-region.amazonaws.com/account-id/quene-name> --dest <https://sqs-region.amazonaws.com/account-id/quene-name>

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Outputs the contents of each SQS message replayed.
    -V, --version    Print version information

OPTIONS:
    -d, --dest <https://sqs-region.amazonaws.com/account-id/quene-name>
            The destination SQS url.

    -m, --max-messages <10>
            The maximum number of messages to replay.

    -r, --region <us-west-2>
            The AWS region

    -s, --source <https://sqs-region.amazonaws.com/account-id/quene-name>
            The source SQS url.

    -x, --dedup-regex <id":"(.*?)">
            A regex applied to each message. The regex must contain a single capture group.
            The value captured identifies the message. Messages identified by a previously seen
            identifier will be deleted, but not replayed.
            
            Avoids replaying duplicate messages.
            
            e.g --selector-regex 'id":"(.*?)"'

```

## Dedup Messages

sqs-replay can deduplicate messages. To avoid replaying logically similar messages, sqs-replay accepts a `dedup-regex`. 
The `dedup-regex` must contain a single capture group that selects the message's deduplication identifier.
Subsequent messages that contain the same deduplication identifier will be deleted, but not replayed.

```shell
sqs-replay \ 
  --source 'https://host/MyDLQueue' \ 
  --dest 'https://host/MyQueue' \ 
  --max-messages 200 
  --dedup-regex 'id":"(.*?)"'
  --verbose
```

The `deup-regex`, `id":"(.*?)"` matches the value of `id` in a json structure. 

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

The `--deup-regex` will replay message 1 and 2, and delete 3.

## License 
MIT
