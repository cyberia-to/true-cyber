// ---
// tags: cyber, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cyber — product face of the soft3 chaosnet (spacepussy-test).
//!
//! The network is soft3 (`soft3 node` = cybergraph+bbg). This binary is the
//! thin product client: install stays light; the stack runs on the node.

use std::env;
use std::process;

const DEFAULT_RPC: &str = "https://cyb.ai/spacepussy-test";
const CHAIN_ID: &str = "spacepussy-test";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "" | "status" => {
            println!(
                "cyber {}  ·  {CHAIN_ID}  ·  soft3 chaosnet",
                env!("CARGO_PKG_VERSION")
            );
            println!("  rpc  {DEFAULT_RPC}");
            println!("  run  cyber sync");
            println!("  node soft3 node   # run the stack locally");
        }
        "sync" => cmd_sync(rpc_from_args(&args[1..])),
        "network" | "net" => {
            println!("network {CHAIN_ID}");
            println!("  role     soft3 chaosnet (cybergraph+bbg)");
            println!("  rpc      {DEFAULT_RPC}");
            println!("  status   {DEFAULT_RPC}/status");
            println!("  denom    testpussy");
            println!("  node     soft3 node --bind 127.0.0.1:7780");
        }
        "help" | "--help" | "-h" | "?" => print_help(),
        "version" | "--version" | "-V" => {
            println!(
                "cyber {} (true-cyber) · soft3 network {CHAIN_ID}",
                env!("CARGO_PKG_VERSION")
            );
        }
        "query" | "tx" | "keys" | "start" | "tendermint" => {
            eprintln!("this is true-cyber (soft3 product face), not cosmos go-cyber.");
            eprintln!("  cyber sync          # spacepussy-test");
            eprintln!("  soft3 node          # run soft3 stack node");
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
            if matches!(
                name,
                "space-pussy" | "spacepussy" | "pussy" | "sp" | "bostrom" | "boot"
            ) {
                eprintln!("`{name}` is cosmos bootloader — not soft3 spacepussy-test.");
                eprintln!("product: {CHAIN_ID} @ {DEFAULT_RPC}");
                process::exit(2);
            }
        }
        i += 1;
    }
    rpc
}

fn cmd_sync(rpc: String) {
    let base = rpc.trim_end_matches('/');
    println!("cyber sync · {CHAIN_ID}");
    println!("  rpc              {base}");
    match probe(base) {
        Ok(s) => {
            println!("  reachable        yes");
            if !s.engine.is_empty() {
                println!("  engine           {}", s.engine);
            }
            if !s.chain_id.is_empty() {
                println!("  chain_id         {}", s.chain_id);
            }
            if !s.moniker.is_empty() {
                println!("  moniker          {}", s.moniker);
            }
            println!("  latest_height    {}", s.latest_height);
            if !s.bbg_root.is_empty() {
                println!("  bbg_root         {}", s.bbg_root);
            }
            if s.signals > 0 || s.particles > 0 {
                println!("  signals          {}", s.signals);
                println!("  particles        {}", s.particles);
                println!("  axons            {}", s.axons);
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
            println!("  which cyber && cyber version   # expect true-cyber");
            println!("  curl -sS {base}/status | head");
            println!("  cargo install soft3 true-cyber --force");
            process::exit(1);
        }
    }
}

struct Status {
    chain_id: String,
    moniker: String,
    engine: String,
    latest_height: u64,
    bbg_root: String,
    signals: u64,
    particles: u64,
    axons: u64,
    catching_up: bool,
}

fn probe(base: &str) -> Result<Status, String> {
    let url = format!("{base}/status");
    let (code, body) = http_get(&url)?;
    if code >= 500 {
        return Err(format!("http {code} from {url}"));
    }
    parse_status(&body).ok_or_else(|| format!("unparsed status from {url}"))
}

fn parse_status(body: &str) -> Option<Status> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let result = v.get("result").unwrap_or(&v);
    let node = result.get("node_info");
    let sync = result.get("sync_info");
    let soft = result.get("soft3");
    Some(Status {
        chain_id: node
            .and_then(|n| n.get("network"))
            .and_then(|x| x.as_str())
            .unwrap_or(CHAIN_ID)
            .into(),
        moniker: node
            .and_then(|n| n.get("moniker"))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .into(),
        engine: node
            .and_then(|n| n.get("engine"))
            .and_then(|x| x.as_str())
            .or_else(|| {
                node.and_then(|n| n.get("protocol"))
                    .and_then(|x| x.as_str())
            })
            .unwrap_or("")
            .into(),
        latest_height: json_u64(sync.and_then(|s| s.get("latest_block_height"))).unwrap_or(0),
        bbg_root: sync
            .and_then(|s| s.get("bbg_root"))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .into(),
        signals: json_u64(soft.and_then(|s| s.get("signals"))).unwrap_or(0),
        particles: json_u64(soft.and_then(|s| s.get("particles"))).unwrap_or(0),
        axons: json_u64(soft.and_then(|s| s.get("axons"))).unwrap_or(0),
        catching_up: sync
            .and_then(|s| s.get("catching_up"))
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
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
        "cyber {} — product face of soft3 spacepussy-test",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  cargo install true-cyber");
    println!("  cyber sync                 # probe soft3 node on cybernode");
    println!("  cyber network");
    println!();
    println!("  the network is soft3:");
    println!("    cargo install soft3");
    println!("    soft3 node --bind 127.0.0.1:7780");
    println!();
    println!("  public rpc: {DEFAULT_RPC}");
    println!("  docs: https://cyber.page/soft3/docs/launch");
}
