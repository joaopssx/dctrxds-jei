#!/usr/bin/env bash
# The Linux quality gate: the template's flow (stage, check, test, build) inside
# WSL. Everything lives on Linux's own disk under ~/dctrxds-gate: nothing is
# written to the Windows checkouts, and no game or player data is touched.
#
# Run it from Windows:
#   wsl -d Ubuntu -- bash tools/wsl_gate.sh
#
# It needs git, a C toolchain, pkg-config and Python 3.12+:
#   wsl -d Ubuntu -u root -- apt-get install -y build-essential pkg-config python3 git
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
gate="$HOME/dctrxds-gate"
pin_of() { python3 -c "import json;print(json.load(open('$here/compatibility.json'))['$1'])"; }
horizon_pin="$(pin_of horizon_revision)"
mixins_pin="$(pin_of horizon_mixins_revision)"
rust_version="$(sed -n 's/^channel = "\(.*\)"/\1/p' "$here/rust-toolchain.toml")"

if ! command -v rustup >/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
        sh -s -- -y --profile minimal --default-toolchain "$rust_version"
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env"
rustup toolchain install "$rust_version" --profile minimal --component rustfmt --component clippy

mkdir -p "$gate"
# Clones of the sibling checkouts, which stay as they are.
# The checkouts belong to the Windows user, so git needs telling they are safe.
trusted=(-c safe.directory='*')
[ -d "$gate/horizon" ] || git "${trusted[@]}" clone "$here/../horizon" "$gate/horizon"
[ -d "$gate/horizon-mixins" ] || git "${trusted[@]}" clone "$here/../horizon-mixins" "$gate/horizon-mixins"
git "${trusted[@]}" -C "$gate/horizon" fetch -q origin
git "${trusted[@]}" -C "$gate/horizon-mixins" fetch -q origin
git -C "$gate/horizon" cat-file -e "$horizon_pin^{commit}"
git -C "$gate/horizon-mixins" checkout -q "$mixins_pin"

# The mod itself, as committed plus any local edits, on Linux's disk.
mkdir -p "$gate/mod"
tar -C "$here" --exclude=./target --exclude=./.git -cf - . | tar -C "$gate/mod" -xf -
cd "$gate/mod"

export HORIZON_MIXINS="$gate/horizon-mixins"
cargo fetch --locked --manifest-path "$HORIZON_MIXINS/Cargo.toml"

run() { echo; echo "=== $*"; "$@"; }
run cargo fmt --all --check
run rustfmt --edition 2024 --check mixins/dctrxds_jei/*.rs
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run python3 tools/horizon.py stage --horizon "$gate/horizon"
run cargo fetch --locked --manifest-path target/mixins/modded/Cargo.toml
run python3 tools/horizon.py check --horizon "$gate/horizon"
run python3 tools/horizon.py test --horizon "$gate/horizon"
run python3 tools/horizon.py build --horizon "$gate/horizon"
echo
echo "Native test output:"
grep -E '^test |^test result' target/horizon-test.log || true
echo "LINUX GATE PASSED"
