# In-process model host

`pivot-host` connects portable Protobuf messages to `pivot-core`. It owns no
transport, daemon lifecycle, persistent store, or global mutable calculation cache.
It is a separate crate so the core and message schema remain independent.

## Registry

Construct `ModelHost::new(invocation_namespace)`, then call
`register(model_ref, snapshot, port_mapping)`. The mapping is a `BTreeMap` from
public interface labels to caller-assigned portable port IDs. Every public port
must be mapped exactly once; unused mappings and duplicate portable IDs fail.
Metadata and optional defaults are derived from the actual snapshot interface,
not accepted as an independent potentially inconsistent description.

Registration requires an exact external model/revision pair. Existing entries
cannot be replaced; edits require a new external revision ID. Entries retain
immutable snapshots and survive changes to or destruction of working models.
Different revisions of a model may coexist. Portable port IDs must stay associated
with the same process-local public port within this live registry. Renaming its
label is safe; rebinding its external ID to another internal port is rejected.

The embedder assigns model/revision/port IDs and a unique invocation namespace.
The host allocates diagnostic invocation IDs by a checked local sequence within
that namespace. No allocation across restarts, durable publication, or registry
reconstruction policy is established. No results are retained or retrievable.

## Sessions and authorization

Construct `Session::new(limits, authorizer)` with explicit policy. `Authorizer`
receives Discover, Describe, or Invoke and a portable model reference. Discovery
filters unauthorized entries; describe/invoke deny before registry lookup, avoiding
model-existence disclosure. Protocol validation happens before engine interaction.
The provided AllowAll policy is an explicit trusted-embedding choice, not user
authentication. A network host must supply authenticated context separately.

The first successful operation must be Handshake. Ordinary requests beforehand
return HandshakeRequired. Failed handshake attempts leave the session unnegotiated;
successful negotiation cannot be repeated. Every operation then requires v1.0.
Bootstrap errors use stable Response error tags/code numbers with a diagnostic v1.0
envelope, which must not be interpreted as successful negotiation. Invalid binary
messages without trustworthy request IDs use empty correlation.

Each session requires exclusive access during dispatch. A host can be shared across
threads with separate sessions, and each invocation uses fresh calculation-local
state. Concurrent request multiplexing within one session is not implemented.

## Dispatch and results

`dispatch(&self, &mut session, encoded_request)` returns one encoded response.
It bounds incoming bytes before decoding, validates request shape, checks session
state and policy, resolves exact portable revision/ports, and runs named invocation
against the registered snapshot. Decimal conversion uses integer limbs only.

Optional public defaults come from interface declarations. InputBinding.source is
Unspecified in requests and Submitted/Defaulted in effective result inputs. Results
record the values actually passed to the engine; numeric equivalence validates
provenance even if scale or zero sign changes. Public output values/errors do not
contain internal graph IDs. Error messages are deliberately generic; diagnostic
logging with privileged details is future work.

ListModels returns revisions in registry key order, up to the page limit. This
implementation's opaque cursor encodes a filtered index. No consistency across
registry changes is promised. DescribeModel returns actual public metadata.
GetInvocationResult remains unsupported. Invocation IDs are diagnostics only.

## Resource limits and remaining work

Limits default to 1 MiB and 1,024 public inputs/outputs. Embedders may configure
smaller limits (message minimum 256 bytes); the current implementation does not
accept values above its initial safety ceilings. Handshake advertises actual
limits. Incoming/outgoing bytes and invocation entry counts are enforced. Large
responses return ResourceExhausted instead of an oversized result. Small session
budgets can make otherwise registered models unavailable through that session.
These limits do not cap internal engine graph node count.

Snapshot registration and dispatch are synchronous. Adding entries requires mutable
host access; evaluation borrows it immutably. Registry removal/retention, durable
identity, model publication, cancellation/compute budgets, audit storage, framing,
authentication, network transports, and REPL remain future work. Native AOT is
still unverified. The host integration tests run in the existing Cargo CI gate.
