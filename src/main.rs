// ---
// tags: cyber, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cyber — thin product CLI for spacepussy-test.
//!
//! ```text
//! cargo install true-cyber
//! cyber sync
//! ```
//!
//! Crate name is `true-cyber` (crates.io `cyber` is taken). Binary is `cyber`.
//! Intentionally has no soft3/cyb dependency — install stays small and works on
//! rustc 1.74+.

use std::env;
use std::process;

/// Public spacepussy-test edge (cybernode / cyberproxy).
const DEFAULT_RPC: &str = "https://cyb.ai/spacepussy-test";
const CHAIN_ID: &str = "spacepussy-test";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "" | "status" => cmd_status(),
        "sync" => cmd_sync(rpc_from_args(&args[1..])),
        "network" | "net" => cmd_network(),
        "help" | "--help" | "-h" | "?" => print_help(),
        "version" | "--version" | "-V" => {
            println!(
                "cyber {} (true-cyber) · network {}",
                env!("CARGO_PKG_VERSION"),
                CHAIN_ID
            );
        }
        // go-cyber style subcommands → clear redirect
        "query" | "tx" | "keys" | "start" | "tendermint" => {
            eprintln!("this is true-cyber (product CLI), not the cosmos go-cyber daemon.");
            eprintln!("  product:  cyber sync          # spacepussy-test");
            eprintln!("  chain:    go-cyber / pussy     # bostrom · space-pussy bootloader");
            process::exit(2);
        }
        other => {
            eprintln!("unknown `{other}` — try `cyber help`");
            process::exit(2);
        }
    }
}

fn rpc_from_args(args: &[String]) -> String {
    let mut rpc = DEFAULT_RPC.to_string();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--rpc" {
            i += 1;
            if let Some(u) = args.get(i) {
                rpc = u.clone();
            }
        } else if args[i] == "--network" || args[i] == "-n" {
            i += 1;
            let name = args.get(i).map(|s| s.as_str()).unwrap_or("");
            if is_bootloader(name) {
                eprintln!("`{name}` is a cosmos bootloader chain — not spacepussy-test.");
                eprintln!("product: {CHAIN_ID} @ {DEFAULT_RPC}");
                process::exit(2);
            }
            if !matches!(
                name,
                "spacepussy-test" | "test" | "soft3" | "sptest" | "default" | ""
            ) {
                eprintln!("unknown network `{name}` (use spacepussy-test)");
                process::exit(2);
            }
        }
        i += 1;
    }
    rpc
}

fn is_bootloader(s: &str) -> bool {
    matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "space-pussy" | "spacepussy" | "pussy" | "sp" | "bostrom" | "boot"
    )
}

fn cmd_status() {
    println!("cyber {}  ·  {CHAIN_ID}", env!("CARGO_PKG_VERSION"));
    println!("  rpc  {DEFAULT_RPC}");
    println!("  run  cyber sync");
}

fn cmd_network() {
    println!("network {CHAIN_ID}");
    println!("  role     soft3 chaosnet (product default)");
    println!("  rpc      {DEFAULT_RPC}");
    println!("  status   {DEFAULT_RPC}/status");
    println!("  health   {DEFAULT_RPC}/health");
    println!("  denom    testpussy");
    println!("  prefix   pussy");
    println!("  (product default · cybernode edge)");
}

fn cmd_sync(rpc: String) {
    let base = rpc.trim_end_matches('/');
    println!("cyber sync · {CHAIN_ID}");
    println!("  rpc              {base}");

    match probe(base) {
        Ok(s) => {
            println!("  reachable        yes");
            if !s.chain_id.is_empty() {
                println!("  chain_id         {}", s.chain_id);
            }
            if !s.moniker.is_empty() {
                println!("  moniker          {}", s.moniker);
            }
            println!("  latest_height    {}", s.latest_height);
            if s.earliest_height > 0 {
                println!("  earliest_height  {}", s.earliest_height);
            }
            println!(
                "  catching_up      {}",
                if s.catching_up { "yes" } else { "no" }
            );
        }
        Err(e) => {
            println!("  reachable        no");
            println!("  detail           {e}");
            println!();
            println!("checks:");
            println!("  1. which cyber     # must be ~/.cargo/bin/cyber from true-cyber");
            println!("  2. cyber version   # expect: cyber x.y (true-cyber)");
            println!("  3. curl -sS {base}/status | head");
            println!("  4. cargo install true-cyber --force");
            process::exit(1);
        }
    }
}

struct Status {
    chain_id: String,
    moniker: String,
    latest_height: u64,
    earliest_height: u64,
    catching_up: bool,
}

fn probe(base: &str) -> Result<Status, String> {
    // try several shapes — trailing slash, /status, /health
    let candidates = [
        format!("{base}/status"),
        format!("{base}/status/"),
        format!("{base}/"),
        base.to_string(),
        format!("{base}/health"),
    ];

    let mut last_err = String::from("no response");
    for url in &candidates {
        match http_get(url) {
            Ok((code, body)) if code < 500 => {
                if url.contains("health") && body.trim() == "ok" {
                    return Ok(Status {
                        chain_id: CHAIN_ID.into(),
                        moniker: String::new(),
                        latest_height: 0,
                        earliest_height: 0,
                        catching_up: false,
                    });
                }
                if let Some(s) = parse_status(&body) {
                    return Ok(s);
                }
                // non-json 2xx still counts as reachable if path was /status-ish
                if code == 200 && url.contains("status") {
                    return Ok(Status {
                        chain_id: CHAIN_ID.into(),
                        moniker: String::new(),
                        latest_height: 0,
                        earliest_height: 0,
                        catching_up: false,
                    });
                }
                last_err = format!("http {code} from {url} (unparsed body)");
            }
            Ok((code, _)) => {
                last_err = format!("http {code} from {url}");
            }
            Err(e) => {
                last_err = format!("{url}: {e}");
            }
        }
    }
    Err(last_err)
}

fn parse_status(body: &str) -> Option<Status> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let result = v.get("result").unwrap_or(&v);
    let node = result.get("node_info");
    let sync = result.get("sync_info");
    let chain_id = node
        .and_then(|n| n.get("network"))
        .and_then(|x| x.as_str())
        .unwrap_or(CHAIN_ID)
        .to_string();
    let moniker = node
        .and_then(|n| n.get("moniker"))
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let latest_height = json_u64(sync.and_then(|s| s.get("latest_block_height"))).unwrap_or(0);
    let earliest_height = json_u64(sync.and_then(|s| s.get("earliest_block_height"))).unwrap_or(0);
    let catching_up = sync
        .and_then(|s| s.get("catching_up"))
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    Some(Status {
        chain_id,
        moniker,
        latest_height,
        earliest_height,
        catching_up,
    })
}

fn json_u64(v: Option<&serde_json::Value>) -> Option<u64> {
    let v = v?;
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn http_get(url: &str) -> Result<(u16, String), String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(12))
        .call()
        .map_err(|e| format!("{e}"))?;
    let code = resp.status();
    let body = resp.into_string().map_err(|e| format!("body: {e}"))?;
    Ok((code, body))
}

fn print_help() {
    println!(
        "cyber {} — spacepussy-test product CLI",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  cargo install true-cyber");
    println!("  cyber sync                 # probe public chaosnet");
    println!("  cyber network              # endpoints");
    println!("  cyber version");
    println!();
    println!("  default rpc: {DEFAULT_RPC}");
    println!();
    println!("  if install fails:  rustup update stable && rustc --version  # need 1.74+");
    println!("  if wrong binary:   which cyber && cargo install true-cyber --force");
    println!("  if sync fails:     curl -sS {DEFAULT_RPC}/status | head");
    println!();
    println!("  not the cosmos go-cyber `cyber` binary (bostrom / space-pussy bootloader).");
}
