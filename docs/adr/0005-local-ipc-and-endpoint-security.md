# ADR-0005: Local IPC framing and endpoint security

Status: Proposed
Date: 2026-10-18

## Context

Pivot needs its first external daemon/client path while retaining transport-free
engine and host APIs. Local endpoints require peer restrictions and bounded reads
even when callers run as the same user.

## Decision

Propose Windows byte-mode named pipes and filesystem Unix-domain sockets on
Linux/macOS. No TCP or shared memory. Each frame is a four-byte unsigned
little-endian payload length followed by one Protobuf message. Reject zero-length
and oversized frames before payload allocation. One connection owns one negotiated
session and exchanges sequential request/response frames.

Propose a protected Windows pipe DACL granting only the current user's SID, remote
client rejection, exclusive first-instance creation, and both-side peer process
token user checks. Propose a current-user-owned mode-0700 Unix directory and
mode-0600 socket, plus both-side OS peer-credential UID checks. Fail closed on
security setup/identity failures. Never silently delete an existing socket or
fall back to a weaker endpoint.

Apply finite frame deadlines, a daemon connection cap, and a conservative host
calculation-work budget before evaluation. Keep authorization as a host policy,
not an assumption that same-user callers are trustworthy.

## Alternatives considered

- TCP loopback: adds authentication and port-management concerns unnecessarily.
- Default Windows pipe ACL: may grant broader access than intended.
- Unix permissions without peer checks: insufficient identity verification.
- Shared memory: premature for this integration slice.

## Consequences

Platform-specific security code is isolated in pivot-transport. Tokio supplies
async I/O, deadlines, signals, and task cleanup; Windows native security calls have
documented unsafe boundaries. Cross-platform source does not imply verified runtime
support. The initial demo is exposed only to the current user; a process running
as that user can still submit hostile requests or impersonate a same-user service.
Administrative/root privileges can bypass ordinary user isolation.

The daemon generates a random startup invocation namespace, hosts one immutable
demo revision, and stores no results. Shutdown stops accepting and cancels/joins
connection tasks; forced termination can leave a Unix socket requiring explicit
cleanup. Persistent model management and publication remain separate future layers.

## Verification / reconsideration trigger

Require real daemon/client round trips, malformed/partial frame survival, session
tests, concurrent clients, endpoint exclusivity, ACL inspection, and shutdown tests.
Windows, Ubuntu WSL2 Linux, and macOS ARM64 are verified by the existing local
CI gates; see [platform validation](../platform-validation.md). An actual
other-user denial test remains open. This validation does not accept the ADR.
Review framing, permissions, and budget behavior before accepting this ADR.
