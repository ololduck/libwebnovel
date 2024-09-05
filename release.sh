#!/bin/bash
set -e
# TODO: add git branch check
cargo test
cargo clippy -r --all-targets --locked
cargo semver-checks check-release --all-features
cargo smart-release --execute --update-crate-index
git push
