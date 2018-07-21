#!/usr/bin/env zsh

set -e

cargo build --release
cp -f ./target/release/batch ./target/holy-web
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 ./target/holy-web

scp ./scripts/init.sql hedlx:~/holy_crackers_web
scp ./target/holy-web hedlx:~/holy_crackers_web/holy-web.new
ssh hedlx 'mv ~/holy_crackers_web/holy-web{.new,}'
