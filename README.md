---
title: true-cyber
tags: cyber, cli
crystal-type: source
crystal-domain: cyber
alias: cyber cli, true-cyber
---
# true-cyber

`cyber` is the headless native graph client. It uses the same GraphSession,
robot registry, custody and signed delivery as cyb. A robot can attach several
neurons; a process or data directory is not another signing subject.

Build this migration from the compatible sibling checkout set with Rust 1.95+:

```sh
cargo install --path ~/cyber/true-cyber --locked
cyber status
cyber neuron list
```

This is a full graph/storage client. The local workspace build and feature set
are distinct from previously published crates; `cargo install true-cyber` from
the registry does not install these uncommitted migration changes.

Choose the network ID from your trusted genesis and its matching signed endpoint:

```sh
cyber neuron create desktop NETWORK_HEX
cyber sync --network NETWORK_HEX --rpc http://127.0.0.1:7780
cyber link topic answer --network NETWORK_HEX --rpc http://127.0.0.1:7780
cyber relay --network NETWORK_HEX --rpc http://127.0.0.1:7780
```

`NETWORK_HEX` is a complete native 32-byte ID, not the transport name or URL.
`neuron attach KEY_FILE NETWORK_HEX` reuses existing custody; `watch` and
`watch-foreign` add observation-only identities. Creation is explicit. `--attachment`
selects an existing binding for an action; `neuron use` changes the UI selection.
Revoke clears that selection but retains original action receipts/history.

A link prints its request ID before publication. Retry the same input with
`--request REQUEST_HEX`, or read `receipt REQUEST_HEX --network NETWORK_HEX`.
Unknown responses preserve the original signed bytes; a changed payload conflicts.
`--amount` and `--valence` remain supported. Author labels such as `--neuron 01`
cannot authorize a write. Endpoint acceptance and local graph roots do not claim
network finality or consensus verification.

`CYBER_HOME`, or `--home DIR`, retains the old default
`~/.cyber/spacepussy-test`. Stop old writers before first opening an old log.
The strict common importer preserves all original frames and retires the append
path to `log/source` plus a manifest; the migrated database is `log.bbg`.
New homes use `log/` directly as their database. No imported author gets invented
keys or an inferred network binding. Malformed sources fail before activation.

```sh
cyber legacy inspect
cyber legacy export NEW_FILE
```

The export is byte-for-byte and refuses to overwrite an existing file.
`cyber help` lists commands and bounds are defined in
[the native client contract](specs/native-client.md).

Validation: `cargo test --test native_client --offline` runs real CLI processes
against actual authenticated soft3 HTTP nodes, including two networks/keys,
unknown results, revocation, admission recovery and legacy log import/export.
Evidence belongs to [the convergence audit](../soft3/audit/neuron-cell/implementation.md).

The `cyber` binary name also exists in the old Cosmos go-cyber daemon. Check
`cyber version` when selecting a binary; foreign chain state/schema is unchanged.

License: cyber — don't trust. don't fear. don't beg.
