---
title: true-cyber
tags: cyber, cli
crystal-type: source
crystal-domain: cyber
alias: cyber cli, true-cyber
---
# true-cyber

The network repository of cyber, the network of CYB: its genesis and its configuration, the input [cybernode](https://github.com/cyberia-to/cybernode) takes to run it. Beside them stand the network's site and its documents. The repository also still carries `cyber`, the command line of a neuron.

cyber is the third network and arrives last, after [space-pussy](https://github.com/cyberia-to/space-pussy) and [bostrom](https://github.com/cyberia-to/bostrom) have proven the machine. It buries no chain: it starts empty, reads the proven states of the two, and mints CYB by claims — every TOCYB of bostrom becomes 666 CYB.

## what is here

| path | what it is |
|---|---|
| [network.toml](network.toml) | the network's identity and rules, as far as they are decided: the name, the coin, claims only, the cap and the schedule of emission |
| [genesis/](genesis/) | how block 0 is made: the states it reads, the claims, the empty graph, and the lock, not built yet |
| [endpoints.toml](endpoints.toml) | where the network is met: the site |
| [site/](site/) | [cyb.money](https://cyb.money): one page, static, no build |
| [audit/](audit/) | receipts |
| [docs/](docs/) | the history, the economy of CYB, what can be run |
| `src/`, `tests/`, `Cargo.toml` | the client `cyber`, until it moves to cybernode |

The layout is the standard of a network repository, cybernode [`specs/network.md`](https://github.com/cyberia-to/cybernode/blob/main/specs/network.md).

## where it stands

Nothing of cyber runs. The network is not launched, the node knows no network by this name, the claim program is not specified, and the genesis is not built: the lock is empty and says so. What is decided is in `network.toml`; what is proposed and what is open is in [the economy of CYB](docs/economics.md). The phase that makes this repository the node's input is N4 of [networks as repositories](https://github.com/cyberia-to/soft3/blob/release/phase1/projects/network-repos.md).

| | |
|---|---|
| the coin | cyb |
| the cap | 18,446,744,069,414,584,321, the order of the Goldilocks field |
| claimable at genesis | 187,416,084,623,451,570 CYB, about 1 % of the cap |
| the graph at genesis | empty |

## the client

`cyber` is the command line of the phase-1 root chain
([soft3/specs/chain.md](../soft3/specs/chain.md)): the three things a neuron
can do — transfer, cyberlink, claim — signed with the key your spell derives
and submitted to a chain node over the HTTP surface of
[chain-wire §7](../soft3/specs/chain-wire.md). Encoding and signing are
`soft3::wallet`, shared with cyb; this binary is the command line around it.

```sh
cargo install --path ~/cyber/true-cyber --locked
cyber neuron new                      # mints ~/.cyber/spell (0600); or reuse cyb's ~/cyb/spell
cyber status --rpc https://cyb.ai/pussy
cyber balance
cyber tx transfer pussy1…-or-64-hex 1000
cyber tx link "what lives?" "the graph" --valence 1
cyber tx claim                        # address-only genesis balance → your neuron
```

Every transaction is NSIG1 over `hemera("soft3/chain/sign/v1\0" ‖ chain ‖ tx)`:
the chain id (the node's `/status` `genesis` field, or `--chain-id`) is inside
the signature, so nothing signed for pussy is valid on bostrom. The nonce is
asked from `/nonce/{neuron}` unless `--nonce` says otherwise; the fee is the
floor (1 unit per byte: 224 / 257 / 140) unless `--fee` says more. `cyber tx …`
prints the txid, then polls `/tx/{txid}` until the transaction is in a block
(`--wait SECS`, `--no-wait`); `--json` prints the receipt.

A bostrom or pussy holder whose pubkey the old chain saw signs with the same
words (the implicit bridge of chain.md §2.8). An account that never sent a
transaction is parked under a dormant owner; `cyber tx claim` moves every
token of it to `neuron_of(pubkey)`.

`cyber help` lists every command and option. Validation:
`cargo test` runs the binary against a mock node that verifies each submitted
transaction with `soft3::wallet::verify` and rejects wrong nonce, wrong chain
id and forged keys.

The `cyber` binary name also exists in the old Cosmos go-cyber daemon. Check
`cyber version` when selecting a binary.

License: cyber — don't trust. don't fear. don't beg.
