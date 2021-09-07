use async_trait::async_trait;
use futures::future::join_all;
use sqs::Region;

type Callback = fn(String);

struct RecvOpts {
    source: String,
    // max_num_messages: Option<i32>,
}

struct SendOpts {
    source: String,
    dest: String,
    messages: Vec<sqs::model::Message>,
    callback: Callback,
}

pub struct ReplayOpts {
    pub source: String,
    pub dest: String,
    pub max_num_messages: Option<i32>,
    pub callback: Callback,
}

#[async_trait]
pub trait Replayer {
    async fn replay(&self, opts: ReplayOpts) -> Result<(), sqs::Error>;
}

pub struct SqsReplayerOpts {
    pub region: Option<String>,
}
pub struct SqsReplayer {
    client: sqs::Client,
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
        })
    }
    async fn receive_message(
        &self,
        opts: RecvOpts,
    ) -> Result<Vec<sqs::model::Message>, sqs::Error> {
        let result = self
            .client
            .receive_message()
            .queue_url(opts.source)
            .send()
            .await?;

        let messages = match result.messages {
            Some(o) => o,
            None => Vec::new(),
        };

        Ok(messages)
    }
    async fn send_messages(&self, opts: SendOpts) -> Result<(), sqs::Error> {
        let messages = opts.messages.clone();

        let mut ss = Vec::new();
        for m in opts.messages {
            let body = m.body.unwrap_or_default();
            let f = self
                .client
                .send_message()
                .message_body(body)
                .queue_url(&opts.dest)
                .send();
            ss.push(f);
        }

        let mut ds = Vec::new();
        for m in messages {
            let body = m.body.unwrap_or_default();
            let handle = m.receipt_handle.unwrap_or_default();
            let d = self
                .client
                .delete_message()
                .receipt_handle(handle)
                .queue_url(&opts.source)
                .send();
            ds.push(d);
            (opts.callback)(body);
        }

        join_all(ss).await;
        join_all(ds).await;

        Ok(())
    }
}

#[async_trait]
impl Replayer for SqsReplayer {
    async fn replay(&self, opts: ReplayOpts) -> Result<(), sqs::Error> {
        let max = opts.max_num_messages.unwrap_or(-1);
        let mut i = 0;
        while max == -1 || i < max {
            let messages = self
                .receive_message(RecvOpts {
                    source: opts.source.clone(),
                })
                .await?;

            if messages.len() == 0 {
                break;
            }

            self.send_messages(SendOpts {
                source: opts.source.clone(),
                dest: opts.dest.clone(),
                messages,
                callback: opts.callback,
            })
            .await?;

            i = i + 1;
        }
        Ok(())
    }
}
