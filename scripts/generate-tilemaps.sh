#! /usr/bin/env bash

set -e
set -u
set -o pipefail

# This script generates tilemap entries

TILESET=${1}

FROM=${2}
TO=${3:-${FROM}}

for i in $(seq "${FROM}" "${TO}"); do
    cat <<EOF
[[tilemaps]]
shape = { rows = 1, columns = 1 }
tiles = [${i}]
tileset = "${TILESET}"

EOF
done
