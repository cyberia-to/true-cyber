// ---
// tags: cyber, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cyber — a small node of the soft3 chaosnet (spacepussy-test).
//!
//! Not a probe: `cyber sync` pulls the peer's signal log (native foculus
//! frames), replays it into a local cybergraph+bbg cell, and recomputes the
//! root itself — verification by recomputation, no trust in served numbers.
//! `cyber link` casts a signal as an encoded frame. HTTP is the last
//! borrowed piece; radio replaces it as transport next.

use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::process;

use cybergraph::{Cybergraph, CyberlinkRecord, NeuronId, Particle, Signal, SELF_NETWORK};
use foculus::{decode_events, encode_signal_frame, CyberFrame};

const DEFAULT_RPC: &str = "https://cyb.ai/spacepussy-test";
const CHAIN_ID: &str = "spacepussy-test";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "" | "status" => cmd_status(),
        "sync" => cmd_sync(rpc_from_args(&args[1..])),
        "link" => cmd_link(&args[1..]),
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
            eprintln!("this is true-cyber (soft3 cell), not cosmos go-cyber.");
            eprintln!("  cyber sync          # replicate + verify spacepussy-test");
            eprintln!("  soft3 node          # run a full soft3 node");
            process::exit(2);
        }
        other => {
            eprintln!("unknown `{other}` — try `cyber help`");
            process::exit(2);
        }
    }
}

// ── the local cell ──────────────────────────────────────────────────────────

/// Where this cyber keeps its slice of the graph ($CYBER_HOME overrides).
fn cell_home() -> PathBuf {
    if let Some(h) = env::var_os("CYBER_HOME") {
        return PathBuf::from(h);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".cyber").join(CHAIN_ID)
}

/// Replay the local log into a fresh graph. Canon: one signal, one block —
/// every applied signal is followed by a finalize, matching the node's own
/// live order and replay exactly.
fn replay(home: &PathBuf) -> (Cybergraph, u64) {
    let mut g = Cybergraph::new();
    let mut signals = 0u64;
    if let Ok(bytes) = std::fs::read(home.join("log")) {
        for frame in decode_events(&bytes) {
            match frame {
                CyberFrame::Signal(s) => {
                    if g.link(s).is_ok() {
                        g.bbg.finalize_block();
                        signals += 1;
                    }
                }
                CyberFrame::Intent(i) => {
                    let _ = g.bbg.apply_intent(&i);
                }
            }
        }
    }
    (g, signals)
}

fn cmd_status() {
    let home = cell_home();
    let (g, signals) = replay(&home);
    println!(
        "cyber {}  ·  {CHAIN_ID}  ·  soft3 cell",
        env!("CARGO_PKG_VERSION")
    );
    println!("  cell   {}", home.display());
    println!(
        "  held   height {} · {} signals · {} particles · {} axons",
        g.bbg.state.height,
        signals,
        g.bbg.state.particles.len(),
        g.bbg.state.axons_out.len()
    );
    println!("  root   {}", hex(&g.bbg.state.root()));
    println!("  rpc    {DEFAULT_RPC}");
    println!("  run    cyber sync · cyber link <from> <to>");
}

// ── sync: pull, replay, verify ──────────────────────────────────────────────

fn cmd_sync(rpc: String) {
    let base = rpc.trim_end_matches('/').to_string();
    println!("cyber sync · {CHAIN_ID}");
    println!("  rpc              {base}");
    let st = match probe(&base) {
        Ok(s) => s,
        Err(e) => {
            println!("  reachable        no");
            println!("  detail           {e}");
            println!();
            println!("  which cyber && cyber version   # expect true-cyber");
            println!("  curl -sS {base}/status | head");
            println!("  cargo install soft3 true-cyber --force");
            process::exit(1);
        }
    };
    println!("  reachable        yes");
    if !st.engine.is_empty() {
        println!("  engine           {}", st.engine);
    }
    if !st.moniker.is_empty() {
        println!("  moniker          {}", st.moniker);
    }

    // pull the peer's new frames into the cell
    let home = cell_home();
    let _ = std::fs::create_dir_all(&home);
    let log_path = home.join("log");
    let held = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    let mut replicated = true;
    match http_get_bytes(&format!("{base}/log?from={held}")) {
        Ok(bytes) if bytes.is_empty() => println!("  pulled           nothing new"),
        Ok(bytes) => {
            let frames = decode_events(&bytes).len();
            if frames == 0 {
                println!("  pulled           {} undecodable bytes — ignored", bytes.len());
            } else {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                {
                    let _ = f.write_all(&bytes);
                }
                println!("  pulled           {frames} frames · {} bytes", bytes.len());
            }
        }
        Err(e) => {
            replicated = false;
            println!("  pulled           none — peer serves no /log ({e})");
        }
    }
    // replay everything we hold; the root is ours, not the peer's word
    let (g, signals) = replay(&home);
    let local_root = hex(&g.bbg.state.root());
    println!("  cell             {}", home.display());
    println!("  height           {}", g.bbg.state.height);
    println!("  signals          {signals}");
    println!("  particles        {}", g.bbg.state.particles.len());
    println!("  axons            {}", g.bbg.state.axons_out.len());
    if !replicated {
        println!("  root(peer)       {}", st.bbg_root);
        println!("  verified         no replication — upgrade the peer to soft3 >=0.8");
        return;
    }
    if st.bbg_root.is_empty() || local_root == st.bbg_root {
        println!("  root             {local_root}");
        println!("  verified         yes — recomputed from {signals} signals locally");
    } else {
        println!("  root(local)      {local_root}");
        println!("  root(peer)       {}", st.bbg_root);
        println!("  verified         NO — replay disagrees with the peer's claim");
        process::exit(1);
    }
}

// ── link: cast a native signal frame ────────────────────────────────────────

fn cmd_link(args: &[String]) {
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if positional.len() < 2 {
        eprintln!("usage: cyber link <from> <to> [--neuron X] [--amount N] [--valence V] [--rpc URL]");
        process::exit(2);
    }
    let from = positional[0];
    let to = positional[1];
    let neuron = flag_value(args, "--neuron").unwrap_or_else(|| "01".into());
    let amount: u64 = flag_value(args, "--amount")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let valence: i8 = flag_value(args, "--valence")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let base = rpc_from_args(args).trim_end_matches('/').to_string();

    // refresh the cell first — step/prev chain against current peer state
    let home = cell_home();
    let _ = std::fs::create_dir_all(&home);
    if let Ok(bytes) = http_get_bytes(&format!(
        "{base}/log?from={}",
        std::fs::metadata(home.join("log")).map(|m| m.len()).unwrap_or(0)
    )) {
        if !bytes.is_empty() && !decode_events(&bytes).is_empty() {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(home.join("log"))
            {
                let _ = f.write_all(&bytes);
            }
        }
    }
    let (g, _) = replay(&home);

    let n = match key32(&neuron) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("bad neuron: {e}");
            process::exit(2);
        }
    };
    let f = key32(from).unwrap_or([0u8; 32]);
    let t = key32(to).unwrap_or([0u8; 32]);
    let (step, prev) = next_pos(&g, &n);
    let signal = Signal {
        neuron: n,
        network: SELF_NETWORK,
        links: vec![CyberlinkRecord {
            neuron: n,
            from: f,
            to: t,
            token: [0u8; 32],
            amount,
            valence,
            height: 0,
        }],
        delta_pi: vec![],
        box_moves: vec![],
        prev,
        step,
        height: 0,
        proof: None,
    };
    let frame = encode_signal_frame(&signal);
    println!("cyber link · {CHAIN_ID}");
    println!("  frame            {} bytes (foculus)", frame.len());
    match http_post_bytes(&format!("{base}/v1/frame"), &frame) {
        Ok(receipt) => {
            for line in receipt.lines() {
                let l = line.trim();
                if l != "---" && !l.is_empty() {
                    println!("  {l}");
                }
            }
        }
        Err(e) => {
            eprintln!("  rejected         {e}");
            process::exit(1);
        }
    }
    // pull our own signal back and re-verify
    cmd_sync(base);
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
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

// ── identity (same rules as the node: hex or hemera-hash of the label) ──────

fn key32(s: &str) -> Result<Particle, String> {
    if let Some(p) = parse_hex32(s) {
        return Ok(p);
    }
    let h = hemera::hash(s.as_bytes());
    let b = h.as_bytes();
    let mut out = [0u8; 32];
    out[..b.len().min(32)].copy_from_slice(&b[..b.len().min(32)]);
    Ok(out)
}

fn parse_hex32(s: &str) -> Option<Particle> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() || s.len() > 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let padded = format!("{s:0>64}");
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn next_pos(cg: &Cybergraph, neuron: &NeuronId) -> (u64, Particle) {
    match cg.chains.get(neuron) {
        Some(chain) if !chain.entries.is_empty() => {
            let step = chain.entries.len() as u64;
            let prev = chain.entries[&(step - 1)].hash();
            (step, prev)
        }
        _ => (0, [0u8; 32]),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ── status wire (cybermark particle; legacy JSON tolerated) ─────────────────

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

// ── http (the last borrowed transport — radio replaces it) ──────────────────

fn http_get(url: &str) -> Result<(u16, String), String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(12))
        .call()
        .map_err(|e| format!("{e}"))?;
    let code = resp.status();
    let body = resp.into_string().map_err(|e| format!("body: {e}"))?;
    Ok((code, body))
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .map_err(|e| format!("{e}"))?;
    let mut out = Vec::new();
    resp.into_reader()
        .take(256 * 1024 * 1024)
        .read_to_end(&mut out)
        .map_err(|e| format!("read: {e}"))?;
    Ok(out)
}

fn http_post_bytes(url: &str, bytes: &[u8]) -> Result<String, String> {
    let resp = ureq::post(url)
        .timeout(std::time::Duration::from_secs(30))
        .set("content-type", "application/octet-stream")
        .send_bytes(bytes)
        .map_err(|e| format!("{e}"))?;
    resp.into_string().map_err(|e| format!("body: {e}"))
}

fn print_help() {
    println!(
        "cyber {} — a small node of soft3 spacepussy-test",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  cargo install true-cyber");
    println!("  cyber sync                     # pull frames, replay, verify the root yourself");
    println!("  cyber link <from> <to>         # cast a cyberlink as a native frame");
    println!("  cyber status                   # your local cell");
    println!("  cyber network");
    println!();
    println!("  the full node is soft3:");
    println!("    cargo install soft3");
    println!("    soft3 node --bind 127.0.0.1:7780");
}
