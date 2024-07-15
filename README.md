# Autogram

[![rust](https://img.shields.io/badge/rust-1.78.0-f17d3e.svg)](https://www.rust-lang.org)
[![tdlib](https://img.shields.io/badge/tdlib-1.8.29-blue.svg)](https://github.com/tdlib/td)
[![tdlib-rs](https://img.shields.io/badge/tdlib_rs-1.0.3-fedcba.svg)](https://github.com/FedericoBruzzone/tdlib-rs)

[中文](./README-cn.md) | English

> Important:
>
> It is highly recommended to utilize robot automation for operations, as this application is equivalent to manual operations performed by a real person.

# Features

To use any function, you must first apply for a client at the [Telegram API](https://my.telegram.org/apps) website. It is necessary to use your home broadband for the application; using a proxy will result in an ERROR.

Create a new folder named `autogram` anywhere, and then copy the [docker-compose.yml](./docker-compose.yml) file into this folder. Modify the environment variable fields, ensuring that `API_ID` and `API_HASH` are configured — these are mandatory. Other environment variables can be optionally configured after logging in. To proceed, execute the command.
```bash
docker compose pull
docker compose run --rm -it autogram login            # Log in to your account, where `API_ID` corresponds to the website you've applied for, and `login` is akin to signing into your account on that website. You will need to input your phone number and verify it with a code to log in. Prior to using any other commands, you must first log in.
docker compose run --rm -it autogram chats            # View the IDs and titles of the first few chat groups, which are used for configuring automation. By default, the top 20 are shown, but you can specify using the `--top 50` parameter.
docker compose run --rm -it autogram chat             # Specify a chat ID and message content to send a message. Example: docker compose run --rm -it autogram chat --chat-id='-1234567890123' -m '/checkin'
docker compose run --rm -it autogram listen           # Monitor a chat and send a webhook. Example: docker compose run --rm -it autogram listen --chat-id='-1234567890123'
docker compose run --rm -it autogram multi-listen     # Listening to multiple chats, example: docker compose run --rm -it autogram multi-listen --chat-id='-1234567890123' --chat-id='-9876543210123'
docker compose run --rm -it autogram follow           # Special attention is paid to certain users, for example: docker compose run --rm -it autogram follow --forward-chat-id='-1234567890123' --user-id=12345678
docker compose run --rm -it autogram help             # Default command, executed when starting with `docker compose up`, provides detailed information about the commands available.
```

# Development

Below are two development containers, with all environmental dependencies pre-configured, ready to use upon launch.
- github workspace
- gitpod

Add environment variables by editing the `~/.bashrc` file.
```
export API_ID=12345678
export API_HASH=1234567890abcdef1234567890abcdef
```
Close the terminal and reopen it for the changes to take effect.
```bash
cargo run
```

# Thank

- [tdlib-rs](https://github.com/FedericoBruzzone/tdlib-rs): Rust wrapper around the Telegram Database Library 🦀
- [td](https://github.com/tdlib/td): Cross-platform library for building Telegram clients