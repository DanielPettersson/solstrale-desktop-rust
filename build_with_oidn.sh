#!/usr/bin/env bash

export RUSTFLAGS="-C target-cpu=native"
cargo build --release --features=oidn-postprocessor
