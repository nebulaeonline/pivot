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
quoting; this command mode has no shell expansion, quotes or scripting grammar.
Authoring formula source is forwarded to the daemon as described below. Unknown commands and
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

Opening the REPL creates a v1.0 invocation connection. The first authoring command
opens an additional v1.1 connection, requiring `pivotd --authoring`. Configure at
least two daemon connection slots per authoring REPL; the authoring connection
also consumes one negotiated authoring-session slot. Local command errors and valid request-level rejections do not discard
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

Minimal working-model authoring is now implemented. Persistence, general scripting,
C# executable IPC, GUI and Native AOT remain separate work. Current Windows/Linux
evidence is linked from the [authoring handoff](../handoffs/2026-10-18-minimal-repl-authoring.md).

## Minimal authoring

Start `cargo run -p pivotd -- --demo --authoring`, then open the REPL as above.
The existing invocation-only daemon and noninteractive commands retain their behavior.
Authoring commands are available only in the REPL, including piped stdin.

**Working models are currently memory-only and are lost when pivotd restarts.**
Creating or opening one displays this warning. No operation is called a durable save.

```text
working create ProfitModel
input Revenue = 250000
input Costs = 85000
formula Net = Revenue - Costs
eval Net
port input Sales Revenue required
port input Expenses Costs optional 85000
port output Profit Net
validate
publish
models
invoke ProfitModel Sales=250000
```

`eval Net` returns 165000. Published invocation uses public port labels, while
unpublished evaluation selects authoring nodes. Public optional defaults can differ
from captured input values. `eval Net Revenue=100000 Costs=25000` temporarily
overrides graph inputs without editing them. Formula parsing, name binding,
dependency handling and calculation occur exclusively in the daemon.

| Command | Behavior |
| --- | --- |
| `working list` | Authorized working-model names, portable IDs and revisions |
| `working create NAME` | Create and select an empty model; display MemoryOnly warning |
| `working use NAME_OR_ID` | Resolve uniquely through authorized discovery and open an attachment |
| `working show` | Read pinned nodes, public ports and paged diagnostics |
| `working refresh` | Explicitly reopen the selected model at its current head |
| `input NAME = DECIMAL` | Add a node or replace that named node's definition with a decimal input |
| `formula NAME = SOURCE` | Add or replace a computed definition, including incomplete source |
| `port input LABEL NODE required` | Add a required public input port |
| `port input LABEL NODE optional DECIMAL` | Add a public input port with an explicit default |
| `port output LABEL NODE` | Add a public output port |
| `validate` | Structural validity and diagnostic count at the selected revision |
| `eval NODE[,NODE...] [INPUT=DECIMAL ...]` | Calculate selected nodes with optional temporary overrides |
| `publish` | Publish the selected exact working revision |

Names/labels occupy one whitespace-delimited token. No workbook coordinates or
identity derivation from names is introduced. Node references resolve through
authorized draft reads to portable IDs. Public-port commands initially add ports;
replacement/removal/rename commands are outside this minimal surface.

Formula source is the text following the first `=`. One optional separating space
is removed; other source whitespace is preserved. There is no CLI formula parser
or evaluator. `formula Net = Revenue +` saves an incomplete draft; `working show`
reports its diagnostics and `validate` fails until a later formula command repairs it.
Invalid drafts cannot be evaluated or published. Working edits never alter a
previously published immutable revision.

## Revisions and uncertain mutations

The client retains the selected exact WorkingRevision and submits one typed edit
batch per edit command. Every mutation gets a fresh operation UUID. It never
implicitly refreshes the expected revision after another client edits. A
RevisionConflict leaves local selection unchanged; inspect the situation and
explicitly `working refresh` before resubmitting an edit.

`reconnect` closes both connections and clears selection/attachments while retaining
any uncertain mutation's operation ID and original typed intent. Use `working use`
to reopen a resource; resources are not owned by connections. If a mutation response
was lost, follow this sequence instead of issuing a replacement command:

```text
reconnect
working recover
```

`working recover` queries the retained operation ID without resending the mutation.
A committed result restores its returned revision/result; a recorded rejection
resolves the pending operation. Unknown retains the original intent because it
does not prove that the operation was never applied.

`working retry` explicitly resends the same operation ID and identical typed intent
with a fresh request ID. It never invents a new mutation ID. New mutations are
blocked while an outcome remains unresolved. Unauthorized/Internal mutation
responses also retain reconciliation context because delivery can be denied after
commitment. Read-only commands remain available when the connection is healthy.

`working abandon` explicitly forgets that pending client context. It does not undo
anything on the daemon and must not be used as a shortcut for retry. Changing
endpoints is blocked while a mutation is pending. A changed service instance cannot
recover or replay an old memory-only operation; the client requires explicit abandonment.
Pending intent is session-local, so resolve it before exiting the REPL.

Requests, responses and page traversal honor negotiated limits. Oversized edits
fail before transmission/receipt tracking; the client does not split a mutation or
truncate formula source. Reads may fail when one entry cannot fit the message cap.
Permission and revision checks remain authoritative on the daemon.
