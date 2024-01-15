use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::builder::GetMessages;
use crate::{DiscordConfig, OAIConfig};

async fn thread_manager(ctx: Context, msg: Message, discord_config: DiscordConfig, oaiconfig: OAIConfig) {
    let mut messages: Vec<ChatCompletionRequestMessage> = vec![];
    // If the thread is inside the allowed channels

    if discord_config.channels.contains(&msg.thread.unwrap().parent_id.unwrap_or_default().get()) {


        let builder = GetMessages::new().limit(10);
        let thread_messages = msg.channel_id.messages(&ctx.http, builder).await.expect("A");
        // Convert messages into prompts
        for message in thread_messages.iter() {
            if message.author.id.get() == discord_config.bot_id {
                let assistant_message = ChatCompletionRequestAssistantMessageArgs::default()
                    .content(&message.content)
                    .build().expect("Couldn't make Assistant message");
                messages.push(assistant_message.into());
            } else {
                let user_message = ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("{}: {}", message.author.name, message.content))
                    .build().expect("Couldn't make user message");
                messages.push(user_message.into());
            }
        }
        // Append system prompt
        let system_message = ChatCompletionRequestSystemMessageArgs::default()
            .content(&oaiconfig.system_prompt)
            .build().expect("Couldn't make system prompt");
        messages.push(system_message.into());
    }
}