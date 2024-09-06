#!/bin/bash
set -e
if [[ "$(git branch --list | grep '*' | cut -d' ' -f2)" != "main" ]]; then
  echo "ERROR: please run this script on the \`main\` branch"
  exit 1
fi
cargo test --all-features
cargo doc --no-deps -r --all-features
cargo clippy -r --all-targets --all-features --locked
cargo semver-checks check-release --all-features
cargo readme >README.md && git add README.md
cargo smart-release --execute --update-crates-index
git push
