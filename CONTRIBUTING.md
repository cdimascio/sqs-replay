# Contributing


## Develop
- Clone the repo
- Build the code

```shell
cargo build
```

- Run the code

```shell
 cargo run  -- -s https://sqs.us-west-2.amazonaws.com/<AWS-ACCOUNT-ID>/<SQS-SOURCE-QUEUE-NAME> -d https://sqs.us-west-2.amazonaws.com/<AWS-ACCOUNT-ID>/<SQS-DEST-QUEUE-NAME>
```

## Package

```shell 
cargo build --release
cd target/release
mv main sqs-replay
tar -czf sqs-replay-mac.tar.gz sqs-replay
gh release create v0.9.0 
gh release upload v0.9.0 sqs-replay-mac.tar.gz
shasum -a 256 sqs-replay-mac.tar.gz
# da7ea07186e32e773ee620d3910ac7cba9b9ea1d9c9a7970d10ddf6c9b63dcb3
# Update tap 
```