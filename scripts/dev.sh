#!/bin/sh
set -eu
. "$(dirname -- "$0")/rust-env.sh"
sh scripts/build.sh
printf 'Игра: http://127.0.0.1:8027\n'
exec python3 -m http.server 8027 --bind 127.0.0.1 --directory dist
