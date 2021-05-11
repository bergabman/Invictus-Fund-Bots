# Invictus Fund Bots
Discord price bots to follow Invictus Capital token value movements, and to provide information from the funds.

Check out the `botconfig_example.toml` file to see what configuration options you can use and what information you have to provide for the bot after you compiled it.

Modules:
    `base`: basic pricebot for invictus funds with invictus api calls.
    `c10`: `base` module on steroids, commands added to get information from funds.
    `icap`: uniswap api based pricebot for ICAP only.
    `invictus_api`: invictus api calls library , used in `base` and `c10` mudules.

To compile it to a raspberry pi 3B+, use Cross in the workspace and choose the module you want to compile. For the simple pricebot, you can use `-p base`.

`cross build -p <module> --target aarch64-unknown-linux-gnu --release`
