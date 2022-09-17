# Latest doesn't work for some reason.
FROM ubuntu:20.04

RUN apt-get update && \
    apt-get install -y build-essential curl

RUN apt-get update

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# Install Go
RUN \
  mkdir -p /goroot && \
  curl https://storage.googleapis.com/golang/go1.19.1.linux-amd64.tar.gz | tar xvzf - -C /goroot --strip-components=1

# Set environment variables.
ENV GOROOT /goroot
ENV GOPATH /gopath
ENV PATH $GOROOT/bin:$GOPATH/bin:$PATH

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get install -y make nodejs
RUN apt-get install -y git

RUN git config --global --add safe.directory '*'

RUN apt-get install -y python3 cmake

RUN apt-get install --assume-yes --no-install-recommends \
    g++-mips-linux-gnu \
    libc6-dev-mips-cross

RUN ~/.cargo/bin/rustup target add mips-unknown-linux-gnu

ENV CC_mips_unknown_linux_gnu=mips-linux-gnu-gcc \
    CXX_mips_unknown_linux_gnu=mips-linux-gnu-g++ \
    CARGO_TARGET_MIPS_UNKNOWN_LINUX_GNU_LINKER=mips-linux-gnu-gcc

RUN apt install -y python3.8-venv

ENV SHELL=/bin/bash
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash

ENV NODE_VERSION="14.19.0"
RUN bash -c ". $HOME/.nvm/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm use $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && npm install -g yarn"

RUN mkdir /code
WORKDIR /code
