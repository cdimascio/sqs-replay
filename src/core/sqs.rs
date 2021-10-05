use crate::core::error::ReplayError;
use futures::future::join_all;
use futures::FutureExt;
use sqs::model::Message;
use sqs::Region;

pub struct SqsOptions {
    pub(crate) source: String,
    pub(crate) dest: String,
    pub(crate)  region: Option<String>,
}

pub(crate) struct Sqs {
    client: sqs::Client,
    source:String,
    dest: String,
}

type Result<T> = std::result::Result<T, ReplayError>;

impl Sqs {
    pub async fn new(opts: SqsOptions) -> Result<Self> {
        let config = if let Some(region) = opts.region {
            aws_config::from_env().region(Region::new(region))
        } else {
            aws_config::from_env()
        }
        .load()
        .await;

        Ok(Sqs {
            client: sqs::Client::new(&config),
            source: opts.source,
            dest: opts.dest,
        })
    }
    pub(crate)  async fn receive_message(&self) -> Result<Vec<Message>> {
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
    pub async fn send_messages<F>(&self, messages: &Vec<Message>, cb: F) -> Result<()>
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

    pub async fn delete_messages(&self, messages: &Vec<Message>) -> Result<()> {
        let mut ds = Vec::new();
        let empty = &String::from("");
        for m in messages {
            let handle = m.receipt_handle.as_ref().unwrap_or(empty);
            ds.push(
                self.client
                    .delete_message()
                    .receipt_handle(handle)
                    .queue_url(&self.source)
                    .send(),
            );
        }
        // TODO handle delete error
        join_all(ds).await;

        Ok(())
    }

    pub async fn receive_messages(&self, batch: i32) -> Vec<Message> {
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
}
