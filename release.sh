#!/bin/bash
set -e
cargo test
cargo clippy -r --all-targets --locked
cargo semver-checks check-release --all-features
cargo smart-release --execute
