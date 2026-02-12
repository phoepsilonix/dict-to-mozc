#!/usr/bin/env bash
set -euo pipefail

TARGET="${CARGO_BUILD_TARGET:?CARGO_BUILD_TARGET is not set}"

case "$TARGET" in
  x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl)
    FEATURES="use-tcmalloc"
    export CC="gcc"
    export RUSTFLAGS="-C link-arg=-fuse-ld=mold"  # 例: mold で高速リンク
    ;;
  aarch64-unknown-linux-gnu|aarch64-unknown-linux-musl)
    FEATURES="use-mimalloc-rs"
    export CC="aarch64-linux-gnu-gcc"  # クロスなら適切な CC
    export RUSTFLAGS=""
    ;;
  *-apple-darwin)
    FEATURES="use-mimalloc-rs"  # 例
    export RUSTFLAGS=""
    ;;
  *-windows-msvc)
    FEATURES=""
    export RUSTFLAGS=""
    ;;
  *)
    echo "Unsupported target: $TARGET"
    exit 1
    ;;
esac

echo "Building for $TARGET with features: $FEATURES"

cargo build --profile dist --target "$TARGET" --features "$FEATURES" "$@"
