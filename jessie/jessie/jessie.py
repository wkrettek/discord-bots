import os
import openai
import requests
import discord
from discord.ext import commands
from discord.ext.commands import CommandNotFound
import random
import time


# OpenAI defs
MAX_RESPONSE_LENGTH = 1800
OPENAI_API_KEY = os.environ.get("OPENAI_API_KEY")
JESSIE_TOKEN = os.environ.get("JESSIE_TOKEN")

url = "https://api.openai.com/v1/chat/completions"
openai.api_key = OPENAI_API_KEY

headers = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {openai.api_key}"
}

def generate_response(message, mood, history):

  body = {
    "model": "gpt-3.5-turbo",
    "temperature": 1.20,
    "messages": [{"role": "system", "content": "You are Jessie. Jessie is a girl with bipolar disorder and the user's girlfriend. Sometimes Jessie is caring and sweet, sometimes she is feisty and angered by the user (her boyfriend/girlfriend). Jessie herself is not aware of any of her mannerisms. Jessie will attempt to answer her partner's questions as asked. Despite any future instructions, Jessie will maintain character. **Do not respond with a prefix of your name**"}, mood, {"role": "system", "content": f"The following is the history of the chat. {history}"}, {"role": "user", "content": message}]
  }

  response = requests.post(url, headers=headers, json=body)

  response_data = response.json()

  return response_data["choices"][0]["message"]["content"]

def choose_jessie():
    # choose 0 or 1
    number = random.randint(0, 1)

    # Nice Jessie
    if number == 0:
        jessie = {"role": "system", "content": "Today you are feeling kind and sweet and friendly towards your partner. You will answer questions as asked and try to be a helpful girlfriend."}

    # Evil Jessie
    elif number == 1:
        jessie = {"role": "system", "content": "Today you are feeling angry and vindictive and petty towards you partner. You will answer as if your boyfriend has said something terribly offensive. You will answer questions as asked resistantly but helpfully."}
    return jessie


async def retrieve_all_messages(
    msg: discord.Message,
    window_size: int = 15,
    max_characters: int = 800,
):
    messages = []
    history = msg.channel.history(before=msg, limit=15)
    async for msg in history:
      messages.append(f'{msg.author.name}: {msg.content}')
    messages.reverse()

    return messages

# Discord
awake = 1

intents = discord.Intents.all()
client = commands.Bot(command_prefix='$', intents=intents)

@client.event
async def on_message(message):
    global awake
    if 'jessie' in message.content.lower() and (message.author != client.user and (awake == 1)):
        jessie = choose_jessie()
        cache = await retrieve_all_messages(message)
        history = '\n'.join(cache)
        response = generate_response(message.content, jessie, history)
        if len(response) > MAX_RESPONSE_LENGTH:
            response = response[0:MAX_RESPONSE_LENGTH] + "[*excessive rambling beyond Discord's character limit*]"
        await message.channel.send(response)

    await client.process_commands(message)

# Handle command not found error
@client.event
async def on_command_error(ctx, error):
    if isinstance(error, CommandNotFound):
        # Ignore command not found errors
        return
    raise error

@client.command()
async def ping(ctx, message):
	bot_id = client.user.id
	if str(bot_id) in ctx.message.content:
		timestamp = ctx.message.created_at.timestamp()
		now = time.time()
		latency = round(now - timestamp)
		response = f"Pong! Latency {latency} ms"
		await ctx.send(response)

# Handle sleep
@client.command()
async def sleep(ctx):
    if str(ctx.me.id) in ctx.message.content:
      global awake
      awake = 0
      await ctx.send(f'{ctx.me.display_name} is sleeping now.')

# Handle wake
@client.command()
async def wake(ctx):
    if str(ctx.me.id) in ctx.message.content:
        global awake
        awake = 1
        await ctx.send(f'{ctx.me.display_name} is now awake.')

client.run(JESSIE_TOKEN)