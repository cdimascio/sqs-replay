# sqs-replay

## Develop

```shell
cargo build
cargo run -- --source https://host/MyDLQueue --dest https://host/MyQueue -max-messages 2
```

## Package

```shell
cargo build --release
```