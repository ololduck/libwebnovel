#!/bin/bash
set -e
# TODO: add git branch check
cargo test --all-features
cargo doc --no-deps -r --all-features
cargo clippy -r --all-targets --all-features --locked
cargo semver-checks check-release --all-features
cargo readme >README.md && git add README.md
cargo smart-release --execute --update-crates-index
git push
