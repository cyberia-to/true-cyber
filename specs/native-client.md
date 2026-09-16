# Native client and historical log migration

`cyber` is a headless graph client. A process, data directory, endpoint and robot
name are not signing identities. It uses the common cyb-core GraphSession,
Registry, custody, signed delivery and NativeMirror; it has no separate subject
implementation or append journal. It is a full storage client, not an
identity-only SDK. Runtime libraries are transitive through the common host;
there is no renderer or inference engine in this binary.

`CYBER_HOME` (or `--home`) keeps its existing directory convention. Opening the
old `log` uses GraphSession's strict native legacy importer and durable file
retirement. Exact frames, ordering, intents and original authors are retained;
the original file becomes `log/source` with a retirement manifest and the graph
is in `log.bbg`. A fresh home uses the `log` directory as its graph database.
Old writers must be stopped before import. Truncation or invalid chain history
fails closed. No source is deleted or rewritten. `legacy inspect/export` reads
the retained original source; export creates a new file exclusively and syncs it.

The old log contains no authenticated genesis/network pin or public-key custody.
Its authors and SELF_NETWORK bytes remain legacy observations. Import never
attaches those authors, invents their keys, changes their network, or claims that
they signed new envelopes. Local graph roots are projection roots, not network
consensus/finality. NativeMirror preserves endpoint operation/receipt provenance
and validates complete pages before publication. It does not authenticate a
consensus root merely because an endpoint reports one.

Network actions require an explicit full `--network` native ID and matched
`neuron/signed-native/1` capabilities. `--rpc` selects transport only. The default
legacy URL is retained as a convenience, never a network identity. `sync` reads
bounded `/v3/history` pages into the shared graph with a durable per-network
cursor. `neuron` exposes the common explicit create/attach/watch/use/revoke/device
commands. Status, sync and navigation do not create a key.

`link` uses the selected or explicit `--attachment` and full native network pin.
FROM/TO retain the previous particle interpretation (up to 64 hex digits, or
Hemera of a text label). The old `--neuron` author-label shortcut is rejected.
Amount is positive, valence is an i8; both are included in the signed Signal.
The request ID is printed before publication and can be supplied with `--request`.
The complete bytes are admitted, signed and locally committed under the current
grant before delivery. Exact retries compare original payload and scope; changed
input conflicts. They reconcile the original local signal and use the existing
signed delivery contract. They never prepare a fresh sequence for an old request.
A crash after admission can finish that exact admission; a dispatched unknown
uses receipt lookup or exact profile-permitted delivery, never generic reexecution.

`relay` delivers the existing bounded outbox in order. A cursor cannot skip an
unresolved action. `receipt REQUEST` can reconcile/read an old request after
revocation; a subsequent POST still requires its original current grant. Endpoint
acceptance is reported separately from finality. Errors, unknown results and
invalid options exit nonzero. HTTP responses, page count and exports are bounded;
a sync page limit reports its durable prefix and requires another invocation.

The graph currently owns one sequence per native subject across its network
observations, matching the common host profile. Independent incompatible forks
of that same subject require separate homes. Network selection never silently
rebases an already signed attempt onto another lineage.
