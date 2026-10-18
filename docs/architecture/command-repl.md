# Pivot command REPL

Start a demo daemon and open the persistent CLI in separate terminals:

```text
cargo run -p pivotd -- --demo
cargo run -p pivot-cli -- repl
```

Omitting a CLI command also starts the REPL. Use `--endpoint LOCAL_ENDPOINT` at
startup to select another local endpoint. Existing one-shot CLI commands still work.

```text
pivot> help
pivot> models
pivot> describe RestaurantProfit --revision demo-v1
pivot> invoke RestaurantProfit Revenue=250000
Net=165000
Margin=0.66
pivot> invoke RestaurantProfit Revenue=100 Costs=25 --revision demo-v1 --outputs Net
Net=75
pivot> reconnect
pivot> exit
```

Output order follows the public interface/request; invocation also prints the exact
external revision and a diagnostic invocation ID. `describe` shows public input
requirements, optional defaults, output labels, and types. Invocations report
per-output calculation errors and preserve decimal transport without floats.

## Commands and input

Commands are `help`, `models`, `describe MODEL`, `invoke MODEL Name=Decimal ...`,
`reconnect`, and `exit`. Describe/invoke accept `--revision ID`; invoke accepts
`--outputs Label,Label`. Models can be selected by public model name or portable
model ID. Discovery must resolve a unique revision; no implicit latest is chosen.
Revision/output selection is per command, not mutable global selection state.

Arguments are whitespace-delimited. Public labels and numeric inputs do not need
quoting; this command mode has no shell expansion, quotes, general expressions,
transient definitions, model edits, or scripting grammar. Unknown commands and
invalid bindings report errors and leave the REPL usable.

Terminal mode uses rustyline for basic cursor/backspace editing and Up/Down command
history. History is session-local, bounded by the editor's memory-history settings,
and never written to disk. Ctrl+C cancels the current input line; EOF or exit closes
the REPL. Commands already executing are not cancelled by a separate engine API.

Piped stdin uses the same parser and protocol path without terminal editing or
prompts. Output is flushed after each command. A session exits successfully on
exit/EOF even if earlier commands failed; this mode is not a batch-script error
contract. One-shot CLI exit codes remain 0 success, 1 request/connection/argument
failure, and 2 calculation-output failure.

## Sessions and disconnects

Opening the REPL creates one negotiated connection and reuses it for subsequent
commands. Local command errors and valid request-level rejections do not discard
the connection. I/O, framing, or response-validation failure discards it. Commands
while disconnected send nothing and instruct the user to reconnect.

Use `reconnect` to make a new connection and handshake. To change endpoint, use
`reconnect --endpoint LOCAL_ENDPOINT`. Reconnect always discards the previous
connection. Initial connection failure also leaves an offline REPL where help,
reconnect, and exit remain available.

An invocation may have completed before a lost response. The client explicitly
reports that ambiguity and never reconnects or replays the invocation automatically.
Only a later, explicit invoke command performs another calculation. Request IDs
are correlation tokens, not retry/idempotency guarantees.

The daemon default allows five minutes of connection idleness. Once a frame starts,
its five-second partial-read deadline applies independently; writes have five seconds.
After the idle policy expires, reconnect explicitly. Daemon flags adjust these
policies without changing the protocol; see [local IPC](local-ipc.md).

## Verification and remaining scope

Windows full CI passes, including six-second idle recovery and a fake peer that
receives Invoke then drops the response. That test checks one invocation, no automatic
reconnect, and later explicit reconnection to a healthy daemon. Existing noninteractive
checks remain in the same integration harness. Linux Rust/IPC tests also pass.
A Linux pseudo-terminal smoke test verified Up-arrow history and backspace editing.
Windows terminal key handling and the new macOS REPL path need further manual checks.

This is a command REPL over immutable registered models. Editable model sessions,
expression evaluation commands, persistence, general scripting, C# IPC, GUI, and
Native AOT remain separate work.
