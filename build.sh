#!/usr/bin/env bash

if ! [ -x "$(command -v cargo)" ]; then
	curl https://sh.rustup.rs -sSf | sh	
fi

cargo build --release

cp target/release/add-types .bin
chmod +x .bin/add-types
