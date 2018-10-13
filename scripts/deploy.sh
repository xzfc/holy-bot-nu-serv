#!/usr/bin/env zsh

set -e

cargo build --release
cp -f ./target/release/batch ./target/holy-web
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 ./target/holy-web

scp ./scripts/init.sql holy@tg:~/web
scp ./target/holy-web holy@tg:~/web/holy-web.new
ssh holy@tg 'mv ~/web/holy-web{.new,}'
