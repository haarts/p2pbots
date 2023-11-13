from openai import OpenAI

key = "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"
GPT_MODEL = "gpt-3.5-turbo-0613"
client = OpenAI(api_key=key)
completion =  client.chat.completions.create(
    model=GPT_MODEL,
    messages=[
        {
            "role": "user",
            "content": "How do I output all files in a directory using Python?",
        },
    ],
)

print(completion.choices[0].message.content)
