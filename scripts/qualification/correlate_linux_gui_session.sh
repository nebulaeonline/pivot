#!/usr/bin/env bash
# Owner-run from the intended guest desktop terminal. No clipboard or GUI access.
set -eu

stamp=$(date -u +%Y%m%dT%H%M%S%N)
report="$HOME/pivot-preflight/gui-correlation-$stamp"
mkdir -p "$report"
{
    date --iso-8601=seconds
    echo 'Intended VM: Pivot-Ubuntu-x64; owner-correlated Basic Session'
    printf 'Host: '; hostname
    printf 'User: '; id -un
    printf 'Boot: '; cat /proc/sys/kernel/random/boot_id
    echo 'Caller cgroup (metadata only):'
    cat "/proc/$$/cgroup"
    if command -v loginctl >/dev/null; then
        echo 'logind caller-session query (self):'
        if loginctl show-session self -p Id -p User -p Name -p Type -p Class \
            -p Remote -p State -p Seat -p Display -p Leader; then
            echo 'Query succeeded; review correlation before assigning an identity.'
        else
            echo 'Caller session unresolved; do not infer identity from this failure.'
        fi
    else
        echo 'loginctl unavailable; no installation attempted.'
    fi
    # Only this user's named desktop processes; no command lines/environments.
    if command -v pgrep >/dev/null; then
        echo 'Current-user gnome-shell process cgroups:'
        for desktop_pid in $(pgrep -u "$(id -u)" -x gnome-shell || true); do
            printf 'gnome-shell PID: %s\n' "$desktop_pid"
            if [ -r "/proc/$desktop_pid/cgroup" ]; then
                cat "/proc/$desktop_pid/cgroup"
                # Query only a scope explicitly associated with that desktop PID.
                scope=$(sed -n 's|.*/session-\([^/]*\)\.scope.*|\1|p' "/proc/$desktop_pid/cgroup")
                if [ -n "$scope" ] && command -v loginctl >/dev/null; then
                    printf 'Desktop cgroup session query: %s\n' "$scope"
                    loginctl show-session "$scope" -p Id -p User -p Name \
                        -p Type -p Class -p Remote -p State -p Seat -p Display \
                        -p Leader || echo 'Desktop session query failed; identity remains Unknown.'
                else
                    echo 'No queryable logind scope found for this desktop process.'
                fi
            else
                echo 'Process metadata unavailable (possibly exited); retain this result.'
            fi
        done
    else
        echo 'pgrep unavailable; desktop-process correlation not attempted.'
    fi
    echo 'Unresolved session/seat fields remain Unknown pending review.'
    echo 'Clipboard sentinel: NotExecuted; GUI isolation: NotApproved'
} > "$report/correlation.txt" 2>&1
(cd "$report" && sha256sum correlation.txt > SHA256SUMS)
printf 'Local original saved: %s\n' "$report"

share=/mnt/j/coding/pivot
if [ ! -d "$share/scripts/qualification" ]; then
    echo 'Share unavailable. Local original retained; do not rerun to transfer.' >&2
    exit 1
fi
dest="$share/artifacts/guest-gui-isolation/Pivot-Ubuntu-x64/gui-correlation-$stamp"
mkdir -p "$dest"
cp "$report/correlation.txt" "$report/SHA256SUMS" "$dest/"
(cd "$dest" && sha256sum -c SHA256SUMS)
printf 'Report copied and hash verified: %s\n' "$dest"
