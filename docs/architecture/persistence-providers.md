# Persistence providers and recovery direction

Status: Project-owner direction, 2026-10-18. The detailed design is now accepted
in [ADR-0008](../adr/0008-transactional-persistence-and-recovery.md), with
[transactional architecture](transactional-persistence.md),
[API](storage-api-proposal.md), [logical schema](storage-schema-proposal.md),
[SQLite design](sqlite-persistence-design.md),
[failure matrix](persistence-recovery-matrix.md) and
[conformance plan](storage-provider-conformance.md). The bounded
[portable codec/import foundation](portable-storage-foundation.md) is implemented;
no provider, database dependency, migration or runtime durability is implemented.

Design one transactional persistence contract with configurable providers. SQLite
is the first implementation and default local provider, accepting an explicit
filesystem database path. Plan PostgreSQL and MySQL adapters, with room for other
providers that meet the same correctness contract. These are planned capabilities,
not currently implemented or validated backend support.

Service deployment mode and storage selection are independent. A local service
can use an external database; a network-facing daemon can use SQLite on its own
local filesystem. Choose a server database for deployment/concurrency requirements,
not merely because clients connect over a network. SQLite documents this distinction
in [appropriate uses](https://www.sqlite.org/whentouse.html). Sharing a SQLite file
over a network filesystem is not the proposed multi-client architecture: clients
connect to Pivot, and Pivot owns storage access.

```text
Pivot management service
           |
Transactional persistence contract
           |
SQLite / PostgreSQL / MySQL / future providers
```

## Ownership and configuration

Keep provider selection and configuration in the Rust service through the storage
boundary. SQLite accepts a configured filesystem path; server providers accept
provider-specific connection configuration and credentials. Clients continue using
Pivot's protocol and do not open the live database directly. Database credentials
must not become client-visible service metadata or diagnostic text.

Initially, pluggability means compiled Rust adapters selected by configuration.
Dynamic library loading is not required. Keep provider SQL, transaction/isolation
mechanisms and migrations behind the adapter; share Pivot's logical records and
semantic requirements rather than forcing identical SQL across databases.

## Transaction contract

Expose domain transactions such as committing a prepared candidate against an
expected revision together with its operation receipt. Do not expose independent
generic save-model/save-receipt calls that permit a half-commit.

Every provider must preserve expected-revision comparisons, atomic state/receipt
commit, exact typed mutation intent, stable bound references, immutable published
revisions and truthful recovery. Publication data and its successful receipt must
be committed together too. Preserve these guarantees even when each database needs
different locking, conditional updates or transaction retry behavior.

Implement SQLite first, then prove the contract with a second provider before
considering the abstraction settled. Do not build all providers simultaneously.
PostgreSQL and MySQL are planned targets; the second implementation and ordering
remain a design decision. Unsupported providers must be rejected explicitly.

## Service ownership and concurrency

Initially retain one active Pivot service owner per store, including server-database
providers. Define how exclusive ownership is enforced and recovered after failure
before implementation. Choosing PostgreSQL or MySQL does not make multiple daemon
instances safe automatically: working heads, in-memory projections, publication
registries, receipts, policy state and cache invalidation would need coordination.
Multi-instance/clustered serving is separate work, not an implied provider feature.

## Persistence design questions

Use database engines rather than a bespoke storage engine/file format. Pivot still
needs versioned application schemas, migrations and recovery. A persistence ADR should
resolve these concrete questions:

- Database ownership in the Rust service through the storage boundary; clients
  continue using the protocol, not opening live database tables directly.
- Transactionally commit working definitions, name directory, portable identities,
  interfaces, revision and mutation receipt together. Publication metadata and its
  durable outcome must share a transaction too.
- Decide durable authority versus in-memory projection. A disk commit followed by
  failed memory installation must recover without duplicate application or a false
  rollback report. Do not claim database/in-memory atomicity merely because database
  transactions exist.
- Store exact decimal limbs/sign/scale as explicitly encoded data, not REAL. Keep
  future IEEE-754 storage separately typed. Preserve source bytes and draft errors.
- Restore portable **bound** references directly. Source text is not enough to
  reconstruct a pre-rename binding. Define a versioned portable bound-expression
  representation; compiled runtime code can remain replaceable/derived. Do not
  serialize raw Rust handles or make runtime layout the database format.
- Persist immutable published definitions, public port identities and lineage, not
  only mutable working heads. Define startup reconstruction and integrity checks.
- Distinguish durable database/resource identity from the freshly allocated running
  service instance. Decide how old operation IDs can be reconciled after restart
  before expanding current instance-scoped recovery guarantees.
- Define durable receipt retention, client acknowledgement/compaction, schema
  migration, interrupted migration recovery, backup/export and database upgrade
  compatibility. Never silently evict receipts still covered by an advertised promise.
- Choose/test each provider's durability/isolation configuration, contention,
  transaction retry and ambiguous-commit behavior. For SQLite, include journal and
  synchronization policy. Specify filesystem assumptions and crash/power-failure
  expectations. Use short database
  transactions; calculations continue against immutable in-memory captures outside
  write transactions.

## Shared conformance tests and deferred scope

Require every provider to pass the same persistence conformance suite, plus its own
engine-specific failure tests. Cover crashes before/after durable commit, duplicate retry,
startup reconciliation, exact decimal round trips, invalid draft preservation,
stable-reference restoration after rename/name reuse, immutable publication recovery
and migration failure. Exercise competing expected revisions, lost commit responses,
receipt/state consistency, publication recovery and exclusive service ownership.
Do not equate a database transaction retry with permission to replay an ambiguous
client mutation under a fresh operation ID.

The public protocol reports durability and recovery guarantees independently of
provider choice. Installing a database driver does not enable a Durable claim or
cross-restart replay promise. Current runtime behavior remains MemoryOnly.
Cross-platform validation and Native AOT/client deployment remain separate concerns.
No database dependency, database file, provider implementation or schema is introduced
by the authoring-protocol design slice.
