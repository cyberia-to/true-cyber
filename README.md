---
title: true-cyber
tags: cyber, cli
crystal-type: source
crystal-domain: cyber
alias: cyber cli, true-cyber
---
# true-cyber

product CLI for [cyber](https://cyber.page) — binary `cyber`, crate `true-cyber`.

```bash
cargo install true-cyber
cyber sync
```

connects to **spacepussy-test** (soft3 chaosnet) on cybernode:

```
https://cyb.ai/spacepussy-test
```

## expected output

```
cyber sync · spacepussy-test
  rpc              https://cyb.ai/spacepussy-test
  reachable        yes
  chain_id         spacepussy-test
  moniker          cyberproxy-spt
  latest_height    …
  catching_up      no
```

## install / sync troubleshooting

```bash
# 1. use a recent rustc (1.74+ is enough for true-cyber 0.5)
rustup update stable
rustc --version

# 2. force reinstall the product binary
cargo install true-cyber --force

# 3. confirm PATH is the true-cyber binary
which cyber
# expect: …/.cargo/bin/cyber
cyber version
# expect: cyber 0.5.x (true-cyber) · network spacepussy-test

# 4. raw probe
curl -sS https://cyb.ai/spacepussy-test/status | head

# 5. sync
cyber sync
```

if `which cyber` is not under `~/.cargo/bin`, you may be hitting the **cosmos go-cyber** daemon binary (different program). put cargo bin first:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

true-cyber 0.5+ is intentionally thin (ureq + serde_json only). it does **not** pull the soft3/cyb stack — install should finish in under a minute.

## license

cyber license: don't trust. don't fear. don't beg.
