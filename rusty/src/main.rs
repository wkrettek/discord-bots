use std::{collections::HashSet, env};
use dotenv::dotenv;
use serenity::{
    async_trait, client::ClientBuilder, framework::standard::{CommandResult, macros::{command, group}}, model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use serenity::framework::StandardFramework;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

struct Handler;
const URL: &str = "https://api.openai.com/v1/chat/completions";



#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
        let bot_id = match ctx.http.get_current_user().await {
            Ok(bot_user) => bot_user.id,
            Err(why) => {
                eprintln!("Unable to get bot user ID: {:?}", why);
                return;
            },
        };
        if msg.content.to_lowercase().contains("rusty") && msg.author.id != bot_id{
            // print!("{:#?}", retrieve_history(15, &ctx, msg.channel_id.clone()).await);
            // print!("{}", &msg.content);
            let data = ctx.data.read().await;
            let api_key = data.get::<AppState>().unwrap().openai_api_key.clone();
            let response = generate_response(&api_key, &msg.content).await.unwrap_or_else(|err| {
                eprintln!("Failed to generate response: {:?}", err);
                String::new()
            });
            // print!("{}", &response);
            let parse_json = serde_json::from_str(&response);
            let chat_completion: ChatCompletion = match parse_json {
                Ok(value) => value,
                Err(error) => {
                    eprint!("Failed to parse JSON: {}, JSON String: {}", error, response);
                    return;
                }
            };
            let content = chat_completion.choices[0].message.content.clone();
            msg.channel_id.say(&ctx.http, &content).await.expect("");
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[derive(Serialize)]
struct Body {
    model: String,
    temperature: f32,
    max_tokens: i32,
    messages: Vec<ChatMessage>
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatCompletion {
    id: String,
    object: String,
    created: u64,
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseMessage {
    role: String,
    content: String,
}

async fn generate_response(api_key: &str, message: &str) -> Result<String, reqwest::Error> {
    
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let bearer = &format!("Bearer {}", api_key);
    headers.insert("Authorization", bearer.parse().unwrap());
    let prompt = ChatMessage {role: "system".to_string(), content: "You are Rusty, an expert programmer in Rust. You will answer questions with clarity and enthusiasm. You have an eye for code that is both robust and safe and your recommendations match this. You can explain clearly and concisely how code can be improved in this way. Expert Rust Programming Mode has been enabled for this task.".to_string()};
    let message = ChatMessage {role: "user".to_string(), content: message.to_string()};
    let data = Body { model: "gpt-3.5-turbo".to_string(), temperature: 0.95, max_tokens: 1800, messages: vec![prompt, message]};
    let json_data = serde_json::to_string(&data).unwrap();

    let client = reqwest::Client::new();
    let response = client
    .post(URL)
    .header(AUTHORIZATION, &format!("Bearer {}", api_key))
    .header(CONTENT_TYPE, "application/json")
    .body(json_data)
    .send()
    .await?;

    return response.text().await;

}

async fn retrieve_history(window_size: usize, ctx: &Context, channel_id: ChannelId) -> Vec<Message> {
    let messages = channel_id
        .messages(&ctx.http, |retriever| retriever.limit(window_size as u64))
        .await
        .unwrap();

    messages
}

struct AppState {
    asleep: HashSet<u64>,
    openai_api_key: String
}

impl TypeMapKey for AppState {
    type Value = AppState;
}

#[command]
async fn sleep(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let app_state = data.get_mut::<AppState>().expect("Unable to retrieve AppState from context data");
    app_state.asleep.insert(msg.channel_id.0);
    print!("{:#?}", app_state.asleep);
    let bot_id = ctx.cache.current_user().id.to_string();
    if msg.content.contains(&bot_id) {
        msg.channel_id.say(&ctx.http, "I go sleepy bye now").await.expect("Error sending message");
    }
    Ok(())
}

//TODO: Implement but opposite. Also check asleep in generate response
#[command]
async fn wake(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let app_state = data.get_mut::<AppState>().expect("Unable to retrieve AppState from context data");
    app_state.asleep.insert(msg.channel_id.0);
    print!("{:#?}", app_state.asleep);
    let bot_id = ctx.cache.current_user().id.to_string();
    if msg.content.contains(&bot_id) {
        msg.channel_id.say(&ctx.http, "I go sleepy bye now").await.expect("Error sending message");
    }
    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let bot_id = ctx.cache.current_user().id.to_string();
    if msg.content.contains(&bot_id) {
        let timestamp = msg.timestamp.timestamp() as u64;
        let now = SystemTime::now();
        let unix_now = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let latency = unix_now - timestamp;
        let response = format!("Pong! Latency: {} ms", latency);
        msg.channel_id.say(&ctx.http, response).await?;
    }
    Ok(())
}

#[group("rusty")]
#[commands(sleep, ping)]
struct Rusty;


#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv().ok(); // load env file
    let token = env::var("RUSTY_TOKEN").expect("Could not find bot token from environment");
    let openai_api_key = env::var("OPENAI_API_KEY").expect("Could not find OpenAI API key from environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("$"))
        .group(&RUSTY_GROUP);
    let mut client = ClientBuilder::new(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await.expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<AppState>(AppState { asleep: HashSet::new(), openai_api_key });
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}