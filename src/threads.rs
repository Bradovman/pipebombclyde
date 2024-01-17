use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs};
use serenity::all::{GuildChannel};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::builder::GetMessages;
use crate::{DiscordConfig, OAIConfig};

pub async fn thread_manager(
    ctx: &Context,
    msg: &Message,
    guild: GuildChannel,
    discord_config: &DiscordConfig,
    oaiconfig: &OAIConfig,
) -> Vec<ChatCompletionRequestMessage> {
    let mut messages: Vec<ChatCompletionRequestMessage> = vec![];
    // If the thread is inside the allowed channels
    // println!("{}", guild.parent_id.unwrap().get());
    if discord_config.channels.contains(&guild.parent_id.expect("Somehow not thread still?").get()) {

        // Append system prompt
        let system_message = ChatCompletionRequestSystemMessageArgs::default()
            .content(&oaiconfig.system_prompt)
            .build().expect("Couldn't make system prompt");
        messages.push(system_message.into());

        let builder = GetMessages::new().limit(10);
        let thread_messages = msg.channel_id.messages(&ctx.http, builder).await.expect("A");
        // Convert messages into prompts
        for message in thread_messages.iter().rev() {
            // Append corresponding prompts
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

    }

    messages
}