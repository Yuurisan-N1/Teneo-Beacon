#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")" || exit 1
NAME=TeneoBeacon
BOT=teneo-bot
OUT=dist/$NAME.zip
STAGE=/tmp/packzip/$NAME

cargo build --release || exit 1

rm -rf dist /tmp/packzip
mkdir -p "$STAGE/src" "$STAGE/assets" "$STAGE/.github/workflows"

cp Cargo.toml Cargo.lock build.rs Makefile run.sh LICENSE README.md HOWTO.txt \
   config.json .gitignore package.sh "$STAGE/"
cp assets/icon.ico assets/icon.png "$STAGE/assets/"
cp .github/workflows/build.yml "$STAGE/.github/workflows/"

( cd src && find . -name '*.rs' -print0 | while IFS= read -r -d '' f; do
    mkdir -p "$STAGE/src/$(dirname "$f")"
    cp "$f" "$STAGE/src/$f"
  done )

printf '{}\n' > "$STAGE/nodeId.json"
printf '# one proxy per line, for example http://user:pass@host:port\n# an empty file means a direct connection\n' > "$STAGE/proxy.txt"
printf '# one account per line, for example ACCOUNTS_1=mail|token|node_count\n# the node cache is kept in nodeId.json beside this file\n' > "$STAGE/.env"

mkdir -p dist
( cd /tmp/packzip && zip -qr "$OLDPWD/$OUT" "$NAME" )
echo "wrote $OUT"
unzip -l "$OUT" | tail -2
