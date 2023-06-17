#! /usr/bin/env bash

set -e
set -u
set -o pipefail

# This script generates tilemap entries

TILESET=${1}
REMAPABLE=${2}

FROM=${3}
TO=${4:-${FROM}}

for i in $(seq "${FROM}" "${TO}"); do
    cat <<EOF
[[tilemaps]]
shape = { rows = 1, columns = 1 }
tiles = [${i}]
tileset = "${TILESET}"
remapable = ${REMAPABLE}

EOF
done