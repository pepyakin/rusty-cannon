# Latest doesn't work for some reason.
FROM ubuntu:20.04

ENV SHELL=/bin/bash
ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update
RUN apt-get install --assume-yes --no-install-recommends \
    build-essential \
    curl \
    g++-mips-linux-gnu \
    libc6-dev-mips-cross \
    make \
    cmake \
    git \
    python3 python3.8-venv


ENV CC_mips_unknown_none=mips-linux-gnu-gcc \
    CXX_mips_unknown_none=mips-linux-gnu-g++ \
    CARGO_TARGET_MIPS_UNKNOWN_NONE_LINKER=mips-linux-gnu-gcc

#
# Install Rustup and Rust
#
# Use this specific version of rust. This is needed because versions of rust after this one broke
# support for -Zbuild-std for the embedded targets.
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain nightly-2022-05-31 --component rust-src
ENV PATH="/root/.cargo/bin:${PATH}"

#
# Install Go
#
RUN \
  mkdir -p /goroot && \
  curl https://storage.googleapis.com/golang/go1.19.1.linux-amd64.tar.gz | tar xvzf - -C /goroot --strip-components=1
ENV GOROOT /goroot
ENV GOPATH /gopath
ENV PATH $GOROOT/bin:$GOPATH/bin:$PATH

#
# Install NVM
#
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash

#
# Install Node & yarn
#
ENV NODE_VERSION="14.19.0"
RUN bash -c ". $HOME/.nvm/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm use $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && npm install --unsafe-perm -g yarn node-gyp-build"

WORKDIR /code

RUN git config --global --add safe.directory '*'
