use async_trait::async_trait;
use sqs::model::Message;
use sqs::Region;

struct SendOpts {
    messages: Vec<Message>,
}

pub struct ReplayOpts {
    pub max_num_messages: Option<i32>,
}

#[async_trait]
pub trait Replayer {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<(), sqs::Error>
    where
        F: Fn(&Message) + Sync + Send;
}

pub struct SqsReplayerOpts {
    pub source: String,
    pub dest: String,
    pub region: Option<String>,
}

pub struct SqsReplayer {
    client: sqs::Client,
    source: String,
    dest: String,
}

impl SqsReplayer {
    pub async fn new(opts: SqsReplayerOpts) -> Result<Self, sqs::Error> {
        let config = if let Some(region) = opts.region {
            aws_config::from_env().region(Region::new(region))
        } else {
            aws_config::from_env()
        }
        .load()
        .await;
        Ok(SqsReplayer {
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
    async fn send_messages<F>(&self, opts: SendOpts, cb: F) -> Result<(), sqs::Error>
    where
        F: Fn(&Message),
    {
        let mut ss = Vec::new();
        let mut ds = Vec::new();

        let empty = &String::from("");
        for m in opts.messages {
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

#[async_trait]
impl Replayer for SqsReplayer {
    async fn replay<F>(&self, opts: ReplayOpts, cb: F) -> Result<(), sqs::Error>
    where
        F: Fn(&Message) + Sync + Send,
    {
        let max = opts.max_num_messages.unwrap_or(-1);
        let mut i = 0;
        while max == -1 || i < max {
            let messages = self.receive_message().await?;

            if messages.len() == 0 {
                break;
            }

            self.send_messages(SendOpts { messages }, &cb).await?;

            i = i + 1;
        }
        Ok(())
    }
}
