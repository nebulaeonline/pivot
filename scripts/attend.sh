#!/bin/sh
script_dir=$(CDPATH= cd -P "$(dirname "$0")" && pwd) || exit 1
exec python3 "$script_dir/attend_validation.py" "$@"
