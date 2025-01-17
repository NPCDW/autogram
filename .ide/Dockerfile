FROM mcr.microsoft.com/devcontainers/rust:latest

RUN curl -fsSL https://code-server.dev/install.sh | sh
RUN code-server --install-extension MS-CEINTL.vscode-language-pack-zh-hans
RUN code-server --install-extension rust-lang.rust-analyzer
RUN code-server --install-extension tamasfe.even-better-toml
RUN code-server --install-extension fill-labs.dependi
RUN code-server --install-extension njzy.stats-bar

RUN apt-get update && apt-get install -y wget unzip openssh-server git libc++1

ENV LANG C.UTF-8
ENV LANGUAGE C.UTF-8