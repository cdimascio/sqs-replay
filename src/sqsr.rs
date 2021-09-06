use async_trait::async_trait;
use futures::future::join_all;

struct RecvOpts {
    url: String,
    max_num_messages: Option<i32>,
}
struct SendOpts {
    url: String,
    messages: Vec<sqs::model::Message>,
}
pub struct ReplayOpts {
    pub source: String,
    pub dest: String,
    pub region: Option<String>,
    pub max_num_messages: Option<i32>,
}

#[async_trait]
pub trait Replayer {
    async fn replay(&self, opts: ReplayOpts) -> Result<(), sqs::Error>;
}

pub struct SqsReplayer {
    client: sqs::Client,
}

impl SqsReplayer {
    pub async fn new() -> Result<Self, sqs::Error> {
        let shared_config = aws_config::load_from_env().await;
        println!("{:?}", shared_config.region());
        Ok(SqsReplayer {
            client: sqs::Client::new(&shared_config),
        })
    }
    async fn receive_message(
        &self,
        opts: RecvOpts,
    ) -> Result<Vec<sqs::model::Message>, sqs::Error> {
        let max = opts.max_num_messages.unwrap_or(1);
        let result = self
            .client
            .receive_message()
            .queue_url(opts.url)
            .max_number_of_messages(max)
            .send()
            .await?;

        let messages= match result.messages {
            Some(o) => o,
            None => Vec::new(),
        };

        Ok(messages)
    }
    async fn send_messages(&self, opts: SendOpts) -> Result<(), sqs::Error> {
        let mut v = Vec::new();
        let url = opts.url;
        for m in opts.messages {
            let f = self
                .client
                .send_message()
                .message_body(m.body.unwrap_or_default())
                .queue_url(&url)
                .send();
            v.push(f);
        }
        join_all(v).await;

        Ok(())
    }
}

#[async_trait]
impl Replayer for SqsReplayer {
    async fn replay(&self, opts: ReplayOpts) -> Result<(), sqs::Error> {
        let messages = self
            .receive_message(RecvOpts {
                url: opts.source,
                max_num_messages: opts.max_num_messages,
            })
            .await?;

        println!("messages {:?}", messages);

        self.send_messages(SendOpts {
            url: opts.dest,
            messages,
        }).await?;

        Ok(())
    }
}
