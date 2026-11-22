# AGENTS.md — true-cyber

> the doctrine of the home is `~/cyber/AGENTS.md`; it applies here first.

this repository is one network: cyber, the network of CYB. its layout is the standard of cybernode `specs/network.md`; the elder twins are `bostrom` and `space-pussy`. it still carries the code of the client `cyber` (`src/`, `tests/`, `Cargo.toml`), which leaves for cybernode's client commands (soft3 `projects/network-repos.md`, decision 2, phase N4).

## what lives here

- `network.toml` — the network's identity and rules; a draft that states only what is decided
- `genesis/` — the sources (the proven states of pussy and bostrom), the claims, the empty graph, the lock (not built)
- `endpoints.toml` — the site
- `site/` — cyb.money; static files, no build, no Node
- `audit/` — receipts
- `docs/` — the history, the economy of the token, what can be run
- `src/`, `tests/`, `Cargo.toml`, `tools.toml` — the client, until it moves

## rules

- nothing of cyber runs. a file here never says otherwise: a value is decided, proposed or open, and says which.
- `genesis/genesis.lock` is written by the first build, with a receipt in `audit/` of two independent builds.
- `docs/economics.md` follows `cyber/cyber/$CYB.md` of the graph; a figure changes there first.
- the crate `true-cyber` stays resolvable on crates.io and is never yanked.

## checks before a commit

- `python3 -c "import tomllib,sys; [tomllib.load(open(f,'rb')) for f in sys.argv[1:]]" network.toml endpoints.toml genesis/sources.toml genesis/distribution.toml genesis/graph.toml genesis/genesis.lock`
- the client's own tests run on the build host, never on the development machine
