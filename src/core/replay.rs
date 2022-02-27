use crate::core::error::ReplayError;
use crate::core::filter::MessageFilter;
use crate::core::sqs::{Sqs, SqsOptions};
use async_trait::async_trait;
use regex::Regex;
use sqs::model::Message;
use std::cmp;

type Result<T> = std::result::Result<T, ReplayError>;

#[derive(Debug)]
pub(crate) struct ReplayOpts {
    pub max_messages: Option<i32>,
    pub selector: Option<String>,
}

#[async_trait]
pub(crate) trait ISqsReplay {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<()>
    where
        F: Fn(&Message) + Sync + Send;
}

pub(crate) struct SqsReplayOptions {
    pub source: String,
    pub dest: String,
    pub region: Option<String>,
}

pub(crate) struct SqsReplay {
    client: Sqs,
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
            r#"{{ "replayed": {}, "deduped": {} }}"#,
            mf.stats.total, mf.stats.deduped
        );

        Ok(())
    }
}

impl SqsReplay {
    pub async fn new(opts: SqsReplayOptions) -> Result<Self> {
        let client = Sqs::new(SqsOptions {
            source: opts.source,
            dest: opts.dest,
            region: opts.region,
        })
        .await?;
        Ok(SqsReplay { client })
    }
    async fn receive_message(&self) -> Result<Vec<Message>> {
        self.client.receive_message().await
    }

    async fn send_messages<F>(&self, messages: &Vec<Message>, cb: F) -> Result<()>
    where
        F: Fn(&Message),
    {
        self.client.send_messages(messages, cb).await
    }

    async fn delete_messages(&self, messages: &Vec<Message>) -> Result<()> {
        self.client.delete_messages(messages).await
    }

    async fn receive_messages(&self, batch: i32) -> Vec<Message> {
        self.client.receive_messages(batch).await
    }

    fn get_regex_selector(&self, selector: Option<String>) -> Result<Option<Regex>> {
        match selector {
            Some(sel) => match Regex::new(sel.as_str()) {
                Ok(re) => {
                    if re.captures_len() < 2 {
                        Err(ReplayError::BadSelector(String::from(
                            "regex must have at least one capture group.",
                        )))
                    } else {
                        Ok(Some(re))
                    }
                }
                Err(e) => Err(ReplayError::BadSelector(e.to_string())),
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    #[test]
    fn test_capture_group() {
        // no capture group returns 1 capture group
        let sel = String::from("test");
        let r = Regex::new(sel.as_str());
        let re = r.unwrap();
        assert_eq!(re.captures_len(), 1);
    }
}
