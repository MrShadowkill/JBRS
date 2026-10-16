#!/bin/sh
set -eu
jbrs_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$jbrs_dir/bin/jbrs" "$@"
