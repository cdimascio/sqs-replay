use async_trait::async_trait;

pub struct ReplayOpts {
    pub source: String,
    pub dest: String,
}

#[async_trait]
pub trait Replayer {
    async fn replay(&self, opts: ReplayOpts) -> Result<(), sqs::Error>;
}

pub struct SqsReplayer ();

#[async_trait]
impl Replayer for SqsReplayer {
    async fn replay(&self, opts: ReplayOpts) -> Result<(), sqs::Error> {
        println!("hello {} {}", opts.source, opts.dest);
        let shared_config = aws_config::load_from_env().await;
        let client = sqs::Client::new(&shared_config);
        let queues = client.list_queues().send().await?;
        println!("{:?}", queues);
        Ok(())
    }
}


// pub async fn replay(opts: ReplayOpts) -> Result<(), sqs::Error> {
//     println!("hello {} {}", opts.source, opts.dest);
//     let shared_config = aws_config::load_from_env().await;
//     let client = sqs::Client::new(&shared_config);
//     let queues = client.list_queues().send().await?;
//     Ok(())
// }
