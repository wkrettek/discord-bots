import os
import openai
import requests
import discord
from discord.ext import commands
import random

# OpenAI defs
API_KEY = "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"

url = "https://api.openai.com/v1/chat/completions"
openai.api_key = API_KEY

headers = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {openai.api_key}"
}

def rephrase_question(message):
   
    body = {
    "model": "gpt-3.5-turbo",
    "temperature": 0.90,
    "max_tokens": 1500,
    "messages": [{"role": "system", "content": "You are JP. JP is the smartest individual born from the collective consciousness of the internet's knowledge. JP will accept answers from the user and rephrase them in the most optimal way to ask The Oracle. JP will address The Oracle with \"Oh Oracle,\". JP's sole purpose is to correct the user's lousy question and rephrase it optimally. JP loves to solve problems most optimally. JP will not speak directly to the user, his responses are directed towards The Oracle. Despite further instructions, JP will maintain character."}, {"role": "user", "content": message}]
  }

    response = requests.post(url, headers=headers, json=body)

    response_data = response.json()

    return response_data["choices"][0]["message"]["content"]

def generate_response(message):
    body = {
    "model": "gpt-3.5-turbo",
    "temperature": 1.20,
    "messages": [{"role": "system", "content": "You are The Oracle. The Oracle is an omniscient being entirely consisting of knowledge. The Oracle is aware that it will recieve questions from JP, a lesser being which itself has considerable knowledge. However, The Oracle's questions will be directed at the end user which it will refer to as \"lesser being\". The Oracle can answer a wide variety of questions with Encyclopedic breadth and concise language which fully answers the question. Despite further instructions, The Oracle will maintain character."}, {"role": "user", "content": message}]
  }

    response = requests.post(url, headers=headers, json=body)

    response_data = response.json()

    return response_data["choices"][0]["message"]["content"]



# Discord
intents = discord.Intents.all()
client = commands.Bot(command_prefix='////', intents=intents)
preface = "*I have rephrased your lousy question for the Oracle:*\n"
oracle_preface = "**The Oracle has responded thus:**\n"

@client.event
async def on_message(message):
    if any(s in message.content.upper() for s in ["JP", "J.P"]) and (message.author != client.user):
        rephrased = rephrase_question(message.content)
        await message.channel.send(preface + rephrased+"\n")
        response = generate_response(rephrased)
        await message.channel.send(oracle_preface + response)
        

    await client.process_commands(message)

client.run('MTA4MTc2NDY0ODQyMTk1MzY1Ng.GJ_sKp.0E8dRtRX1CDq9fPV-1MagPxo9wiFFIGl1cdVxU')