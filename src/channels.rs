
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs};

use serenity::model::channel::Message;

use crate::{DiscordConfig, OAIConfig};


pub async fn reply_chain_to_query (
    msg: &Message,
    discord_config: &DiscordConfig,
    oaiconfig: &OAIConfig,
) -> Vec<ChatCompletionRequestMessage> {
    let mut messages: Vec<ChatCompletionRequestMessage> = vec![];
    let mut current_message = msg;
    let mut chain_length = 0;


    while chain_length < 10{
        match &current_message.referenced_message {
            Some(referenced) => {
                if current_message.author.id.get() == discord_config.bot_id {
                    let assistant_message = ChatCompletionRequestAssistantMessageArgs::default()
                        .content(&current_message.content)
                        .build().expect("Couldn't make Assistant message");
                    messages.insert(0, assistant_message.into())
                } else {
                    let user_message = ChatCompletionRequestUserMessageArgs::default()
                        .content(format!("{}: {}", current_message.author.name, current_message.content))
                        .build().expect("Couldn't make user message");
                    messages.insert(0, user_message.into());
                }


                current_message = &referenced;
            },
            None => chain_length = 10        }

        chain_length += 1;
    }

    let system_message = ChatCompletionRequestSystemMessageArgs::default()
        .content(&oaiconfig.system_prompt)
        .build().expect("Couldn't make system prompt");
    messages.insert(0, system_message.into());

    messages
}