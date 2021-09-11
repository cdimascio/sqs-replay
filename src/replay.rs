use async_trait::async_trait;
use sqs::model::Message;
use sqs::Region;

pub struct ReplayOpts {
    pub max_messages: Option<i32>,
}

#[async_trait]
pub trait ISqsReplay {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<(), sqs::Error>
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
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<(), sqs::Error>
    where
        F: Fn(&Message) + Sync + Send,
    {
        let max = opts.max_messages.unwrap_or(-1);
        let mut i = 0;
        while max == -1 || i < max {
            let messages = self.receive_message().await?;

            if messages.len() == 0 {
                break;
            }

            self.send_messages(messages, &cb).await?;

            i = i + 1;
        }
        Ok(())
    }
}

impl SqsReplay {
    pub async fn new(opts: SqsReplayOptions) -> Result<Self, sqs::Error> {
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
    async fn receive_message(&self) -> Result<Vec<Message>, sqs::Error> {
        let result = self
            .client
            .receive_message()
            .queue_url(&self.source)
            .send()
            .await?;

        let messages = match result.messages {
            Some(o) => o,
            None => Vec::new(),
        };

        Ok(messages)
    }
    async fn send_messages<F>(&self, messages: Vec<Message>, cb: F) -> Result<(), sqs::Error>
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

            let handle = m.receipt_handle.as_ref().unwrap_or(empty);
            let d = self
                .client
                .delete_message()
                .receipt_handle(handle)
                .queue_url(&self.source)
                .send()
                .await;
            ds.push(d);
            ss.push(s);
            cb(&m);
        }

        Ok(())
    }
}
