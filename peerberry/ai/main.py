from openai import OpenAI
import argparse

def main():
    parser = argparse.ArgumentParser(description='A script to read portfolio and market data, based on that makes an investment (or not)')

    parser.add_argument('--cash', type=float, help='Cash available to invest', required=True)
    parser.add_argument('--prefix', type=str, help='The prefix of the to be read CSV files', required=True)
    parser.add_argument('--verbose', action='store_true', help='Enable verbose output')

    args = parser.parse_args()

    if args.verbose:
        print(f"Potentially investing {args.cash} EUR, using {args.prefix} as a prefix")

    with open(f"{args.prefix}_portfolio.csv", 'r') as file:
        portfolio = file.read()
    with open(f"{args.prefix}_loans.csv", 'r') as file:
        market = file.read()
    with open(f"{args.prefix}_investments.csv", 'r') as file:
        secondary_market = file.read()

    instructions = '''
        Keep the portfolio diversified. Don't put all the money in the
        same country or same originator or all the funds in a single
        loan. In general go for the investment with a high discount on
        the secondary market. Higher interest is preferred. When there's
        no attractive investment is it fine not to invest. Long term
        (1 year or longer) loans are not attractive. Perhaps there is
        more information available in the market and secondary market,
        please take that information into consideration too.
    '''

    if args.verbose:
        print(f"These are the instructions to the AI:\n\n {instructions}")

    content = f"""
        You are my portfolio manager, these are your instructions:

        {instructions}

        This is the portfolio you are currently managing. It is a CSV file:
    
        {portfolio}

        You have a maximum of {args.cash} EUR to invest.
    
        These loans are available on the market (also CSV):
    
        {market}
    
        These alternative investments are available on the secondary market (also CSV):
    
        {secondary_market}

        Tell me which loans you would invest given the current portfolio.
        For each loan you would invest in tell me how much, partial
        investments are allowed. Don't exceed the amount of money
        available ({args.cash} EUR) for investment!

        For each selected loan please give a short summary of the most 
        relevant aspects of the loan.
        """

    if args.verbose:
        print(f"This is the entire prompt:\n\n {content}")

    key = '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'
    # GPT_MODEL = 'gpt-3.5-turbo-1106'
    GPT_MODEL = 'gpt-4-1106-preview'
    client = OpenAI(api_key=key)
    completion =  client.chat.completions.create(
        model=GPT_MODEL,
        messages=[
            {
                'role': 'user',
                'content': content,
            },
        ],
    )
    print(completion.choices[0].message.content)


if __name__ == "__main__":
    main()


