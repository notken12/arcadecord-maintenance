use std::env;

use serenity::async_trait;
use serenity::model::gateway::{Activity, Ready};

use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content =
                "🛠 Bot is under maintenance, please check back in a couple minutes and try again.";

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!(
            "Shard id {} running {} is connected!",
            ctx.shard_id,
            ready.user.tag()
        );
        ctx.shard
            .set_activity(Some(Activity::playing("🛠 maintenance")));
    }
}

fn get_shard_range(shard_manager_id: u64, shard_manager_count: u64, total_shards: u64) -> [u64; 2] {
    let low = shard_manager_id * total_shards / shard_manager_count;
    let high = (shard_manager_id + 1) * total_shards / shard_manager_count - 1;
    [low, high]
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("BOT_TOKEN").expect(
        "Expected a token in the environment. Set BOT_TOKEN environment variable to the bot token.",
    );
    let shard_manager_count: u64 = env::var("SHARD_MANAGER_COUNT").expect("Expected a shard manager count in the environment. Set SHARD_MANAGER_COUNT to the total amount of shard manager processes.").parse().expect("Shard manager count must be a 64 bit integer");
    let total_shards: u64 = env::var("TOTAL_SHARDS").expect("Expected amount of total shards in the environment. Set TOTAL_SHARDS to the total amount of shards spawned by all shard managers.").parse().expect("Shard manager count must be a 64 bit integer");
    let shard_manager_pod_prefix = env::var("SHARD_MANAGER_POD_PREFIX").expect("Expected a shard manager pod prefix in the environment. Set SHARD_MANAGER_POD_PREFIX to the prefix of each shard manager's POD_NAME created by Kubernetes.");
    let pod_name = env::var("POD_NAME").expect(
        "Expected a pod name in the environment. Set POD_NAME environment variable to $POD_PREFIX + id of shard manager"
    );
    let shard_manager_id: u64 = pod_name[shard_manager_pod_prefix.len()..]
        .parse()
        .expect("Shard manager id must be an 64 bit integer");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let shard_range = get_shard_range(shard_manager_id, shard_manager_count, total_shards);

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start_shard_range(shard_range, total_shards).await {
        println!("Client error: {:?}", why);
    }
}
