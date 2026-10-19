# Frozen storage codec v1 fixtures

These hex files and `manifest.tsv` are immutable compatibility evidence, not
generated build output. They were initially assembled with an independent small
CBOR encoder and explicit logical records, rather than calling the Rust encoder.
`frozen_v1_codec_compatibility_corpus` checks exact re-encoding and restoration.

Manifest columns are filename, phase (`ok`, `decode`, `restore`) and expected
error. The 11 cases include a working draft with all definition/operator states,
signed zero and a previously unresolved name now present; a valid immutable
publication with u64::MAX sequence; unsupported codec/numeric/expression/operator
versions; cross-model reference; invalid decimal flags; nonminimal CBOR; trailing
bytes; and a structurally invalid publication.

`decode` distinguishes malformed/noncanonical/unsupported portable data from a
validly encoded draft. `restore` tests a validly encoded record that cannot be
installed as an immutable publication. `ok` does not imply a working draft is
executable: preserved parse/bind failures remain valid stored drafts.

No test writes these files. New formats get a new version directory and explicit
reader policy. Existing v1 files must continue decoding with their defined
semantics; changing fixtures is not a migration. See
[layout and limits](../../../../../docs/architecture/portable-storage-foundation.md).
