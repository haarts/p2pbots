# Automate investing in Esketit and Peerberry

## Esketit

This is a largely stalled effort due to the cost of the OpenAI API. At
the time of this commit it costs around $0.10 to make a call. I want to
do these calls several times a day. But even at 1x a day this is too much.

That is not to say the results with GPT-4 are great. What is missing from
this code base is the function calling. This is trivial to add really.

Configuration goes with a simple `config.toml` file:

```toml
username = "your@email.com"
password = "your password"
min_interest_rate = 9
max_term_period = 100
tfa_url = "http://100.112.251.5:3030/esketit"
````

The intend was that the binary would periodically run. It would kick of
another script that would do the interaction with OpenAI and wait until
that script terminates this binary. Very circular. This hasn't been
completed.

PS the API of Esketit is not great. What's with the `X-XSRF-TOKEN`
that is copied from the cookie to a header??

## Peerberry

The program logs in to Peerberry (very pleasant API!) and does some
automated investing following some hard-coded, simple rules. This
basically just works. Some minimal configuration is done via the
`config.toml` file:

```toml
email = "your@email.com"
password = "your password"
max_loan_term = 100
min_interest = 9.0
tfa_url = "http://100.112.251.5/peerberry"
```

I ran this program on a SystemD timer. See the `peerberry.timer` and
`peerberry.service` files.

## 2FA

I assume you are using 2FA for your accounts. For this there's an other
binary which I assume you are running on a private, secure server. I
used a Wireguard network. You need to point the `esketit` and `peerberry`
binary to this server with a config parameter.

There's a `2fa.service` file included for you convenice.
