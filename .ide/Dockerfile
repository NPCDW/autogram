FROM ubuntu:latest

RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf -o rustup.sh
RUN sh rustup.sh -y
RUN bash -c "echo source $HOME/.cargo/env >> /etc/bash.bashrc"

RUN curl -fsSL https://code-server.dev/install.sh | sh
RUN code-server --install-extension MS-CEINTL.vscode-language-pack-zh-hans
RUN code-server --install-extension rust-lang.rust-analyzer
RUN code-server --install-extension tamasfe.even-better-toml
RUN code-server --install-extension fill-labs.dependi
RUN code-server --install-extension njzy.stats-bar
RUN code-server --install-extension Tencent-Cloud.coding-copilot
RUN code-server --install-extension cnbcool.cnb-welcome
RUN code-server --install-extension donjayamanne.githistory

RUN apt-get update && apt-get install -y wget unzip openssh-server git libc++1

ENV LANG C.UTF-8
ENV LANGUAGE C.UTF-8