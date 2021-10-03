use crate::error::ReplayError;
use async_trait::async_trait;
use regex::Regex;
use futures::future::join_all;
use futures::FutureExt;
use sqs::model::Message;
use sqs::Region;
use std::collections::HashSet;

type Result<T> = std::result::Result<T, ReplayError>;

#[derive(Debug)]
pub struct ReplayOpts {
    pub max_messages: Option<i32>,
    pub selector: Option<String>,
}

#[async_trait]
pub trait ISqsReplay {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<()>
    where
        F: Fn(&Message) + Sync + Send;
}

pub struct SqsReplayOptions {
    pub source: String,
    pub dest: String,
    pub region: Option<String>,
}

pub struct SqsReplay {
    client: sqs::Client,
    source: String,
    dest: String,
}

#[async_trait]
impl ISqsReplay for SqsReplay {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<()>
    where
        F: Fn(&Message) + Sync + Send,
    {
        let max = opts.max_messages.unwrap_or(-1);
        let mut rfs = Vec::new();
        let mut i = 0;
        while max == -1 || i < max {
            rfs.push(self.receive_message().boxed());
            i = i + 1;
        }
        let rsv = join_all(rfs).await;
        let rs = self.filter_messages(rsv, opts.selector);

        match rs {
            Ok(messages) => {
                self.send_messages(messages, &cb).await?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl SqsReplay {
    pub async fn new(opts: SqsReplayOptions) -> Result<Self> {
        let config = if let Some(region) = opts.region {
            aws_config::from_env().region(Region::new(region))
        } else {
            aws_config::from_env()
        }
        .load()
        .await;

        Ok(SqsReplay {
            client: sqs::Client::new(&config),
            source: opts.source,
            dest: opts.dest,
        })
    }
    async fn receive_message(&self) -> Result<Vec<Message>> {
        let result = self
            .client
            .receive_message()
            .queue_url(&self.source)
            .send()
            .await;

        if result.is_err() {
            return Err(ReplayError::Sqs(Box::new(result.err().unwrap())));
        }

        let messages = match result.unwrap().messages {
            Some(o) => o,
            None => Vec::new(),
        };

        Ok(messages)
    }
    async fn send_messages<F>(&self, messages: Vec<Message>, cb: F) -> Result<()>
    where
        F: Fn(&Message),
    {
        let mut ss = Vec::new();
        let mut ds = Vec::new();
        let empty = &String::from("");
        for m in messages {
            let body = m.body.as_ref().unwrap_or(empty);
            let s = self
                .client
                .send_message()
                .message_body(body)
                .queue_url(&self.dest)
                .send()
                .await;

            if s.is_err() {
                return Err(ReplayError::Sqs(Box::new(s.err().unwrap())));
            }
            let handle = m.receipt_handle.as_ref().unwrap_or(empty);
            let d = self
                .client
                .delete_message()
                .receipt_handle(handle)
                .queue_url(&self.source)
                .send()
                .await;

            if d.is_err() {
                return Err(ReplayError::Sqs(Box::new(d.err().unwrap())));
            }
            ds.push(d);
            ss.push(s);
            cb(&m);
        }

        Ok(())
    }

    fn filter_messages(
        &self,
        results: Vec<Result<Vec<Message>>>,
        selector: Option<String>,
    ) -> Result<Vec<Message>> {
        let mut seen = HashSet::new();
        let empty = &String::from("");
        if selector.is_some() {
            // if selector is present, use key to proactively delete previously played messages
            let mut filtered_messages = Vec::new();
            let sel = selector.unwrap().clone();
            let mut skipped_count = 0;
            for rm in results {
                match rm {
                    Ok(messages) => {
                        for m in messages {
                            let body = m.body.as_ref().unwrap_or(empty);
                            let re = Regex::new(sel.as_str()).unwrap();
                            match re.captures(body) {
                                Some(c) => {
                                    let key = String::from(c.get(1).unwrap().as_str());
                                    if !seen.contains(&key) {
                                        seen.insert(key);
                                        filtered_messages.push(m);
                                    } else {
                                        skipped_count += 1;
                                    }
                                },
                                None => {}
                            };
                        }
                    }
                    Err(e) => println!("[error] {}", e),
                }
            }
            Ok(filtered_messages)
        } else {
            let mut filtered_messages = Vec::new();
            for r in results {
                match r {
                    Ok(messages) => {
                        for m in messages {
                            filtered_messages.push(m)
                        }
                    }
                    Err(e) => println!("[error] {}", e),
                }
            }
            Ok(filtered_messages)
        }
    }
}
