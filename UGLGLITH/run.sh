#!/bin/sh
set -eu
jbrs_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$jbrs_dir"
export JBRS_SAVE="${JBRS_SAVE:-$jbrs_dir/save.json}"
exec cargo run --release -- "$@"
