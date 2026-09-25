#!/bin/sh
set -eu

repository=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
target=${1:-x86_64-unknown-linux-musl}
skip_build=${2:-}
artifact_name="ams-$target"
destination="$repository/dist/$artifact_name"

case "$destination" in
  "$repository"/dist/*) ;;
  *) printf 'Unsafe package destination: %s\n' "$destination" >&2; exit 1 ;;
esac

if [ "$skip_build" != "--skip-build" ]; then
  cargo build --release --locked --target "$target"
fi

rm -rf -- "$destination"
mkdir -p "$destination/saves" "$destination/mods" "$destination/logs"
cp "$repository/config.toml" "$repository/LICENSE" "$destination/"
cp -R "$repository/data" "$repository/gamemodes" "$destination/"
cp "$repository/target/$target/release/ams" "$destination/ams"
printf '%s\n' "$destination"
