use std::env;
use std::time::Duration;
use std::fs;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serde::{Deserialize, Serialize};
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionResponseMessage, CreateChatCompletionRequestArgs,
};

// Thanks to https://github.com/dnanhkhoa/acm/ for the basis of the oai stuff
#[derive(Serialize, Deserialize)]
struct OAIConfig {
    api_base_url: String,  // The base URL of the Inference API provider
    api_key: String,       // Your API key from the Inference API provider
    model_name: String,    // The ID of the model to use
    system_prompt: String, // The contents of the system prompt
    max_tokens: u16,       // The maximum number of tokens that can be generated
    request_timeout: u64,  // The timeout for the request in seconds
}
#[derive(Deserialize)]
struct ResponseCandidate {
    message: ChatCompletionResponseMessage, // This stores a single commit message candidate generated by the model
}

#[derive(Deserialize)]
struct ResponseCandidates {
    choices: Vec<ResponseCandidate>, // This stores all the commit message candidates generated by the model
}




struct Handler {
    config: OAIConfig,
    http_client: reqwest::Client,
    channels: Vec<u64>,
    bot_id: u64
}
#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message is received - the
    // closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be dispatched
    // simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if self.channels.contains(&msg.channel_id.get()) && msg.author.id.get() != self.bot_id && !msg.content.starts_with("!")  {
            let payload = CreateChatCompletionRequestArgs::default()
                .max_tokens(self.config.max_tokens)
                .model(&self.config.model_name)
                .temperature(0.7)
                .messages([
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(&self.config.system_prompt)
                        .build().expect("Couldn't make system message").into(),
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(format!("{}: {}", msg.author.name,msg.content) )
                        .build().expect("Couldn't make user message").into(),

                ])
                .build()
                .expect("Couldn't make request payload");


            // Send request for inference
            let response = self.http_client
                .post(format!("{}/chat/completions", &self.config.api_base_url))
                .bearer_auth(&self.config.api_key)
                .json(&payload)
                .send()
                .await
                .expect("Failed to send the request to the Inference API provider")
                .error_for_status().expect("Server returned error")
                .json::<ResponseCandidates>()
                .await
                .expect("Failed to parse the response from the Inference API provider");

            let message = response
                .choices
                .first() // Only the first generated commit message is used
                .expect("No messages generated")
                .message
                .content
                .as_ref()
                .expect("No messages generated");




            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, message.to_string().replace("Pipebomb Clyde: ", "")).await {
                println!("Error sending message: {why:?}");
            }
        }
    }



    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    //
    // Openai/Together
    //
    let oai_api_key = std::env::var("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not set");

    //Setup config
    let config = OAIConfig {
        api_base_url: "https://api.together.xyz/v1".to_string(),
        api_key: oai_api_key,
        model_name: fs::read_to_string("model").expect("Unable to read model file"),
        system_prompt: fs::read_to_string("system.prompt").expect("Unable to prompt file"),
        max_tokens: 500,
        request_timeout: 30,
    };

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.request_timeout))
        .build().expect("Failed to make http client");


    //
    // Discord
    //

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Cereate handler for messages.
    let handler = Handler {config, http_client, channels: vec![1194933874371874886, 1195183494666657853], bot_id: 1194929672945934356};

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.




    let mut client =
        serenity::prelude::Client::builder(&token, intents).event_handler(handler).await.expect("Err creating client");



    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}