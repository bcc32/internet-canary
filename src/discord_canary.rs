use chrono::DateTime;
use chrono::Local;
use serenity::Client;
use serenity::model::prelude::*;
use serenity::prelude::*;

struct Handler {
    hostname: String,
    start_time: DateTime<Local>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "!ping" {
            let _ = msg.channel_id.say(&context, "Pong!").await;
            let _ = msg
                .channel_id
                .say(
                    &context,
                    super::info::current(&self.hostname, self.start_time),
                )
                .await;
        }
    }
}

pub struct Config {
    pub token: String,
}

async fn run_forever(config: Config) -> serenity::Result<()> {
    let hostname = hostname::get().unwrap().into_string().unwrap();
    let start_time = Local::now();
    let mut client = Client::builder(&config.token, GatewayIntents::all())
        .event_handler(Handler {
            hostname,
            start_time,
        })
        .await?;

    client.start().await?;

    Ok(())
}

pub fn run_forever_sync(config: Config) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(run_forever(config)).unwrap()
}
