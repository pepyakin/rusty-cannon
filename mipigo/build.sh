#!/usr/bin/env bash
set -e

cd ../minigeth
export GOOS=linux
export GOARCH=mips
export GOMIPS=softfloat
go build -o ../mipigo/minigeth
cd ../mipigo

cp ../arbitrary/arbitrary-prover-main/target/mips-unknown-none/release/arbitrary-prover-main .

python3 -m venv venv

source venv/bin/activate
pip3 install -r requirements.txt
./compile.py arbitrary-prover-main
./compile.py
deactivate
