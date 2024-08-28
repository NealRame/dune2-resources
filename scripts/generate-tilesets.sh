#! /usr/bin/env bash

set -e
set -u
set -o pipefail

# This script generates tilemap entries

TILESET=${1}

WIDTH=${2}
HEIGHT=${3}

FROM=${4}
TO=${5:-${FROM}}

cat <<EOF
[[tilesets]]
id = "${TILESET}"
size = { width = ${WIDTH}, height = ${HEIGHT} }
tile_refs = [
EOF

for i in $(seq "${FROM}" "${TO}"); do
    cat <<EOF
    { index = ${i} },
EOF
done

cat <<EOF
]
EOF
