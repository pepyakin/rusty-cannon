#!/usr/bin/env bash
set -e

cd ../minigeth
export GOOS=linux
export GOARCH=mips
export GOMIPS=softfloat
go build -o ../mipigo/minigeth
cd ../mipigo

cp ../hello-world/target/target/release/hello-world .

python3 -m venv venv

source venv/bin/activate
pip3 install -r requirements.txt
./compile.py hello-world
./compile.py
deactivate
