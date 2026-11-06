#!/usr/bin/env bash
set -u

cd "$(dirname "$0")" || exit 1
BIN=target/release/teneo-bot
SESSION=teneo

case "${1:-run}" in
  build)
    cargo build --release
    ;;
  run)
    [ -x "$BIN" ] || cargo build --release || exit 1
    exec "$BIN" run
    ;;
  probe)
    [ -x "$BIN" ] || cargo build --release || exit 1
    exec "$BIN" probe
    ;;
  selftest)
    [ -x "$BIN" ] || cargo build --release || exit 1
    exec "$BIN" selftest
    ;;
  bg)
    [ -x "$BIN" ] || cargo build --release || exit 1
    screen -dmS "$SESSION" "$BIN" run
    echo "session $SESSION started"
    ;;
  screen)
    screen -r "$SESSION"
    ;;
  stop)
    screen -S "$SESSION" -X quit 2>/dev/null
    echo "session $SESSION stopped"
    ;;
  *)
    echo "usage: ./run.sh build|run|probe|selftest|bg|screen|stop"
    exit 1
    ;;
esac
