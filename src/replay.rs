use crate::error::ReplayError;
use crate::filter::MessageFilter;
use async_trait::async_trait;
use futures::future::join_all;
use futures::FutureExt;
use regex::Regex;
use sqs::model::Message;
use sqs::Region;
use std::cmp;

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

const BATCH_SIZE: i32 = 100;

#[async_trait]
impl ISqsReplay for SqsReplay {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<()>
    where
        F: Fn(&Message) + Sync + Send,
    {
        let re = self.get_regex_selector(opts.selector)?;

        // minus one to account for first message (which doubles as a test)
        let max = opts.max_messages.unwrap_or(i32::MAX) - 1;
        if max < 0 {
            return Ok(());
        }

        let batch = cmp::max(cmp::min(max, BATCH_SIZE), 1);
        let batches = max / batch;
        let mut mf = MessageFilter::new(re.clone());

        // Do the first synchronously (receive, send, delete)
        // This enables capturing errors e.g. permission early without running a full batch
        let messages = self.receive_message().await?;
        mf.add(messages);
        self.send_messages(&mf.results, &cb).await?;

        for _ in 0..batches {
            mf.clear();
            let messages = self.receive_messages(batch).await;
            mf.add(messages);
            self.send_messages(&mf.results, &cb).await?;
            self.delete_messages(&mf.skipped).await?;
        }

        if max > batches * batch {
            mf.clear();
            let batch = max - batches * batch;
            let messages = self.receive_messages(batch).await;
            mf.add(messages);
            self.send_messages(&mf.results, &cb).await?;
            self.delete_messages(&mf.skipped).await?
        }

        println!();
        println!(
            r#"{{ "processed": {}, "deduped": {} }}"#,
            mf.stats.total, mf.stats.deduped
        );

        Ok(())
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
    async fn send_messages<F>(&self, messages: &Vec<Message>, cb: F) -> Result<()>
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

    async fn delete_messages(&self, messages: &Vec<Message>) -> Result<()> {
        let mut ds = Vec::new();
        let empty = &String::from("");
        for m in messages {
            let handle = m.receipt_handle.as_ref().unwrap_or(empty);
            ds.push(self
                .client
                .delete_message()
                .receipt_handle(handle)
                .queue_url(&self.source)
                .send());
        }
        // TODO handle delete error
        join_all(ds).await;

        Ok(())
    }

    async fn receive_messages(&self, batch: i32) -> Vec<Message> {
        //Vec<Result<Vec<Message>>> {
        let mut rfs = Vec::new();
        for _ in 0..batch {
            rfs.push(self.receive_message().boxed());
        }
        let rvs = join_all(rfs).await;

        let mut messages: Vec<Message> = Vec::new();
        for rv in rvs {
            if rv.is_ok() {
                let ms = &mut rv.unwrap();
                messages.append(ms);
            }
            // TODO collect errors
        }
        messages
    }

    fn get_regex_selector(&self, selector: Option<String>) -> Result<Option<Regex>> {
        match selector {
            Some(sel) => match Regex::new(sel.as_str()) {
                Ok(re) => {
                    if re.captures_len() != 2 {
                        Err(ReplayError::BadSelector)
                    } else {
                        Ok(Some(re))
                    }
                }
                Err(..) => Err(ReplayError::BadSelector),
            },
            None => Ok(None),
        }
    }
}
