#!/bin/bash
set -e
# TODO: add git branch check
cargo test
cargo clippy -r --all-targets --locked
cargo semver-checks check-release --all-features
cargo readme >README.md && git add README.md
cargo smart-release --execute --update-crates-index
git push
