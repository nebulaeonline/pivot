#!/usr/bin/env bash
# Owner-run metadata capture only: no clipboard access, GUI launch or approval.
set -eu

stamp=$(date -u +%Y%m%dT%H%M%S%N)
local_report="$HOME/pivot-preflight/gui-session-$stamp"
mkdir -p "$local_report"
{
    date --iso-8601=seconds
    echo 'Intended VM: Pivot-Ubuntu-x64 (owner must correlate with VMConnect)'
    echo 'Connection: VMConnect Basic Session (owner reported; not independently detected)'
    hostname
    id -un
    cat /proc/sys/kernel/random/boot_id
    printf 'session=%s\ntype=%s\ndesktop=%s\ndisplay=%s\nwayland=%s\n' \
        "${XDG_SESSION_ID-}" "${XDG_SESSION_TYPE-}" "${XDG_CURRENT_DESKTOP-}" \
        "${DISPLAY-}" "${WAYLAND_DISPLAY-}"
    if [ -n "${XDG_SESSION_ID-}" ] && command -v loginctl >/dev/null; then
        loginctl show-session "$XDG_SESSION_ID" \
            -p Id -p Name -p Type -p Class -p Remote -p State -p Seat -p Display ||
            echo 'Session query failed; retain output for review.'
    else
        echo 'Session query unavailable: missing XDG_SESSION_ID or loginctl.'
    fi
    echo 'Clipboard sentinel: NotExecuted'
    echo 'GUI isolation: NotApproved'
} > "$local_report/session.txt" 2>&1
printf 'Local report saved: %s\n' "$local_report"

share=/mnt/j/coding/pivot
if [ ! -d "$share/scripts/qualification" ]; then
    echo 'Shared repository unavailable. Local report retained; do not rerun capture just to copy.' >&2
    exit 1
fi
dest="$share/artifacts/guest-gui-isolation/Pivot-Ubuntu-x64"
mkdir -p "$dest"
cp -R "$local_report" "$dest/"
cmp "$local_report/session.txt" "$dest/gui-session-$stamp/session.txt"
printf 'Report copied and verified: %s/gui-session-%s\n' "$dest" "$stamp"
