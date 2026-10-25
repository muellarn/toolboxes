#!/usr/bin/env bash
# rs - run single-file Rust scripts against the precompiled crates in ~/.rust-tools.
#
#   rs script.rs [args...]   run a script (Windows paths such as C:\x\y.rs work too)
#   rs -e 'code' [args...]   run code as the body of `fn main() -> anyhow::Result<()>`
#   rs --warmup              compile all toolbox crates (after `cargo add`)
#   rs --clean               delete cached script copies and their binaries
set -euo pipefail

TOOLS="${RUST_TOOLS:-$HOME/.rust-tools}"
BIN_DIR="$TOOLS/src/bin"
OUT_DIR="$TOOLS/target/script"

usage() { sed -n '4,7p' "$0" | sed 's/^# \{0,1\}//' >&2; exit 2; }

# Build scripts fingerprint environment variables (CC, CFLAGS, PKG_CONFIG_*, ...).
# A fixed environment keeps login shells, plain shells and wsl.exe calls from
# invalidating each other's precompiled crates. Cargo reads .cargo/config.toml
# (mold linker) relative to the working directory. Compiler messages name the
# internal copy src/bin/<name>.rs; the optional second argument replaces that
# path with the script path the caller used.
build() {
  local shown
  shown=$(printf '%s' "${2:-src/bin/$1.rs}" | sed 's/[|&\\]/\\&/g')
  (cd "$TOOLS" && env -i HOME="$HOME" PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
    cargo build -q --profile script --bin "$1" 2>&1 1>&3 \
    | sed "s|src/bin/$1\.rs|$shown|g" >&2) 3>&1
}

[ $# -ge 1 ] || usage
case "$1" in
  --warmup) build _warmup; exit ;;
  --clean)
    rm -rf "$BIN_DIR"/[se]_*.rs "$OUT_DIR"/[se]_* "$OUT_DIR"/deps/[se]_* \
      "$OUT_DIR"/.fingerprint/[se]_* "$OUT_DIR"/incremental/[se]_*
    exit ;;
  -h|--help) usage ;;
  -e)
    [ $# -ge 2 ] || usage
    code=$(printf '#![allow(unused_imports, redundant_semicolons)]\nuse anyhow::{Context, Result, bail};\nfn main() -> anyhow::Result<()> {\n%s\n;\nOk(())\n}\n' "$2")
    name="e_$(printf '%s' "$2" | sha1sum | cut -c1-12)"
    shift 2 ;;
  *)
    script=$1 shown=$1; shift
    if [[ $script =~ ^[A-Za-z]:[\\/] ]]; then script=$(wslpath -u "$script"); fi
    script=$(realpath "$script")
    stem=$(basename "$script" .rs)
    # Path hash keeps scripts with the same file name apart.
    name="s_${stem//[^A-Za-z0-9_]/_}_$(printf '%s' "$script" | sha1sum | cut -c1-8)"
    code=$(cat "$script") ;;
esac

src="$BIN_DIR/$name.rs"
bin="$OUT_DIR/$name"
# Rewrite the copy only when the code changed, so its mtime stays valid for caching.
if [ ! -f "$src" ] || [ "$(cat "$src")" != "$code" ]; then
  printf '%s\n' "$code" > "$src.tmp" && mv "$src.tmp" "$src"
fi
# Skip cargo's fingerprint check (~0.2 s) when the binary is newer than everything it depends on.
if ! { [ "$bin" -nt "$src" ] && [ "$bin" -nt "$TOOLS/Cargo.lock" ] && [ "$bin" -nt "$TOOLS/Cargo.toml" ]; }; then
  build "$name" "${shown:-}"
  touch "$bin"  # cargo leaves an up-to-date binary untouched
fi
exec "$bin" "$@"
