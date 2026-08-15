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

impl Status {
    fn empty() -> Self {
        Status {
            chain_id: String::new(),
            moniker: String::new(),
            engine: String::new(),
            latest_height: 0,
            bbg_root: String::new(),
            signals: 0,
            particles: 0,
            axons: 0,
            catching_up: false,
        }
    }
}

fn probe(base: &str) -> Result<Status, String> {
    let url = format!("{base}/status");
    let (code, body) = http_get(&url)?;
    if code >= 500 {
        return Err(format!("http {code} from {url}"));
    }
    parse_status(&body).ok_or_else(|| format!("unparsed status from {url}"))
}

/// The native wire is a cybermark particle — frontmatter `key: value` lines
/// between `---` fences. Nodes still on soft3 <0.7 answer in JSON; a tiny
/// string extractor keeps them readable through the transition.
fn parse_status(body: &str) -> Option<Status> {
    if body.trim_start().starts_with("---") {
        parse_cybermark(body)
    } else {
        parse_legacy_json(body)
    }
}

fn parse_cybermark(body: &str) -> Option<Status> {
    let mut s = Status::empty();
    let mut in_frontmatter = false;
    let mut fields = 0u32;
    for line in body.lines() {
        let t = line.trim();
        if t == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        let Some((k, v)) = t.split_once(':') else {
            continue;
        };
        let v = v.trim();
        fields += 1;
        match k.trim() {
            "chain" => s.chain_id = v.into(),
            "moniker" => s.moniker = v.into(),
            "engine" => s.engine = v.into(),
            "height" => s.latest_height = v.parse().unwrap_or(0),
            "bbg-root" => s.bbg_root = v.into(),
            "signals" => s.signals = v.parse().unwrap_or(0),
            "particles" => s.particles = v.parse().unwrap_or(0),
            "axons" => s.axons = v.parse().unwrap_or(0),
            "catching-up" => s.catching_up = v == "true",
            _ => fields -= 1,
        }
    }
    (fields > 0).then_some(s)
}

/// Pull the few known fields out of the pre-0.7 JSON status with plain string
/// search — enough for our own emitter, no JSON dependency.
fn parse_legacy_json(body: &str) -> Option<Status> {
    if !body.trim_start().starts_with('{') {
        return None;
    }
    let field = |key: &str| -> Option<String> {
        let probe = format!("\"{key}\"");
        let at = body.find(&probe)? + probe.len();
        let rest = body[at..].trim_start().strip_prefix(':')?.trim_start();
        if let Some(r) = rest.strip_prefix('"') {
            Some(r[..r.find('"')?].to_string())
        } else {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '.')
                .unwrap_or(rest.len());
            Some(rest[..end].to_string())
        }
    };
    let num = |key: &str| field(key).and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut s = Status::empty();
    s.chain_id = field("network").unwrap_or_else(|| CHAIN_ID.into());
    s.moniker = field("moniker").unwrap_or_default();
    s.engine = field("engine").or_else(|| field("protocol")).unwrap_or_default();
    s.latest_height = num("latest_block_height");
    s.bbg_root = field("bbg_root").unwrap_or_default();
    s.signals = num("signals");
    s.particles = num("particles");
    s.axons = num("axons");
    s.catching_up = field("catching_up").as_deref() == Some("true");
    Some(s)
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
