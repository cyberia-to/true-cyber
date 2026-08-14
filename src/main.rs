// ---
// tags: cyber, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cyber — product CLI for the soft3 chaosnet **spacepussy-test**.
//!
//! ```text
//! cargo install true-cyber
//! cyber sync                 # probe spacepussy-test (local soft3 node)
//! cyber network              # print endpoints
//! ```
//!
//! Crate name is `true-cyber` because crates.io `cyber` is taken.
//! Binary on PATH is always `cyber`.
//!
//! Not a wrapper around cosmos bootloader RPC (space-pussy / bostrom on
//! cybernode). Those chains are migration sources — see cyber.page/bootloader.

use soft3::network::{self, Network};
use std::path::{Path, PathBuf};
use std::process::Command;

const ORG: &str = "https://github.com/cyberia-to";

// ── paint ───────────────────────────────────────────────────────────────────

fn paint(code: &str, s: &str) -> String {
    format!("\x1b[{code}m{s}\x1b[0m")
}
fn bold(s: &str) -> String {
    paint("1", s)
}
fn dim(s: &str) -> String {
    paint("2", s)
}
fn green(s: &str) -> String {
    paint("32", s)
}
fn yellow(s: &str) -> String {
    paint("33", s)
}
fn red(s: &str) -> String {
    paint("31", s)
}

// ── network product surface ─────────────────────────────────────────────────

fn parse_global_network(args: &[String]) -> (Network, Vec<String>) {
    let mut net = Network::DEFAULT;
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--network" || args[i] == "-n" {
            i += 1;
            let name = args.get(i).map(|s| s.as_str()).unwrap_or("");
            net = parse_net(name);
        } else {
            rest.push(args[i].clone());
        }
        i += 1;
    }
    (net, rest)
}

fn parse_net(name: &str) -> Network {
    if let Some(n) = Network::parse(name) {
        return n;
    }
    if Network::is_bootloader_name(name) {
        eprintln!(
            "`{name}` is a cosmos bootloader chain on cybernode — not spacepussy-test."
        );
        eprintln!(
            "product network: {} ({})",
            Network::DEFAULT,
            Network::DEFAULT.role()
        );
        eprintln!("  rpc {}", Network::DEFAULT.rpc());
        std::process::exit(2);
    }
    eprintln!("unknown network `{name}` (use spacepussy-test|test|soft3)");
    std::process::exit(2);
}

fn cmd_sync(net: Network) {
    println!("cyber sync · {}", net.chain_id());
    println!("  role             {}", net.role());
    println!("  rpc              {}", net.rpc());
    match network::probe(net) {
        Ok(s) => {
            println!(
                "  reachable        {}",
                if s.reachable {
                    green("yes")
                } else {
                    yellow("degraded")
                }
            );
            if let Some(code) = s.http_status {
                println!("  http             {code}");
            }
            if !s.body_preview.is_empty() {
                println!(
                    "  body             {}",
                    s.body_preview.replace('\n', " ")
                );
            }
        }
        Err(e) => {
            println!("  reachable        {}", red("no"));
            println!("  detail           {e}");
            println!();
            println!("{}", bold("spacepussy-test"));
            println!("  soft3 chaosnet — product default after install.");
            println!();
            println!("{}", dim("not this:"));
            println!(
                "  {}",
                dim("cosmos space-pussy / bostrom on cybernode (bootloader migration sources)")
            );
            std::process::exit(1);
        }
    }
}

fn cmd_network(net: Network) {
    println!("network {}", net.chain_id());
    println!("  role     {}", net.role());
    println!("  prefix   {}", net.bech32_prefix());
    println!("  denom    {}", net.denom());
    println!("  rpc      {}", net.rpc());
    println!("  lcd      {}", net.lcd());
    println!("  index    {}", net.index());
    println!("  ws       {}", net.websocket());
    if net == Network::DEFAULT {
        println!("  (product default)");
    }
}

fn cmd_manifesto() {
    for line in soft3::manifesto() {
        println!("{line}");
    }
}

// ── forward soft3 / cy when present ─────────────────────────────────────────

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn forward(bin: &str, args: &[String], hint: &str) {
    match which(bin) {
        Some(path) => {
            let status = Command::new(path).args(args).status();
            match status {
                Ok(s) => std::process::exit(s.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("cyber: failed to run {bin}: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("  {} `{bin}` not on PATH", yellow("·"));
            eprintln!("  {}", dim(hint));
            eprintln!(
                "  day-one surface: {}",
                green("cyber sync")
            );
            std::process::exit(2);
        }
    }
}

// ── optional toolchain bootstrap (advanced) ─────────────────────────────────

#[derive(serde::Deserialize)]
struct Registry {
    tool: Vec<Tool>,
}

#[derive(serde::Deserialize)]
struct Tool {
    name: String,
    dir: String,
    pkg: Option<String>,
    bin: Option<String>,
    alias: Option<String>,
    desc: String,
}

impl Tool {
    fn bin(&self) -> &str {
        self.bin.as_deref().unwrap_or(&self.name)
    }
    fn repo(&self) -> &str {
        self.dir.split('/').next().unwrap_or(&self.dir)
    }
}

fn registry() -> &'static [Tool] {
    static REG: std::sync::OnceLock<Vec<Tool>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        toml::from_str::<Registry>(include_str!("../tools.toml"))
            .expect("tools.toml is malformed")
            .tool
    })
}

fn tool(name: &str) -> Option<&'static Tool> {
    registry()
        .iter()
        .find(|t| t.name == name || t.alias.as_deref() == Some(name))
}

fn repos() -> Vec<&'static str> {
    let mut rs: Vec<&str> = registry().iter().map(|t| t.repo()).collect();
    rs.push("cyber");
    rs.push("optica");
    rs.push("soft3");
    rs.push("true-cyber");
    rs.sort();
    rs.dedup();
    rs
}

fn cyber_root() -> PathBuf {
    if let Some(r) = std::env::var_os("CYBER_ROOT") {
        return PathBuf::from(r);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Path::new(&home).join("cyber")
}

fn bin_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Path::new(&home).join(".cargo").join("bin")
}

fn sync_one(name: &str, root: &Path) -> bool {
    let target = root.join(name);
    if target.exists() {
        println!("  {} {}  {}", green("●"), bold(name), dim("present"));
        return true;
    }
    let url = format!("{ORG}/{name}.git");
    println!("  {} {}  {}", dim("…"), bold(name), dim(&url));
    let ok = Command::new("git")
        .args(["clone", "--depth", "1", &url])
        .arg(&target)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        println!(
            "  {} {}  {}",
            yellow("○"),
            bold(name),
            dim("unavailable — private repo or no network")
        );
    }
    ok
}

fn cmd_source(names: &[String]) {
    let root = cyber_root();
    let _ = std::fs::create_dir_all(&root);
    let targets: Vec<&str> = if names.is_empty() {
        repos()
    } else {
        names.iter().map(|s| s.as_str()).collect()
    };
    let mut missing = 0u32;
    for name in &targets {
        if !sync_one(name, &root) {
            missing += 1;
        }
    }
    if missing > 0 {
        println!(
            "  {}",
            dim(&format!(
                "{missing} unavailable — the rest of the stack works without them"
            ))
        );
    }
}

fn link_bin(target: &Path, name: &str) -> bool {
    let link = bin_dir().join(name);
    let _ = std::fs::remove_file(&link);
    #[cfg(unix)]
    let res = std::os::unix::fs::symlink(target, &link);
    #[cfg(not(unix))]
    let res = std::fs::copy(target, &link).map(|_| ());
    match res {
        Ok(()) => {
            println!(
                "  {} {} {} {}",
                green("●"),
                bold(name),
                dim("→"),
                dim(&link.display().to_string())
            );
            true
        }
        Err(e) => {
            println!("  {}: link {}: {e}", red("error"), name);
            false
        }
    }
}

fn install_one(t: &Tool, root: &Path) -> bool {
    println!("  {} {}", dim("building"), bold(&t.name));
    let mut cargo = Command::new("cargo");
    cargo
        .arg("build")
        .arg("--release")
        .current_dir(root.join(&t.dir))
        .env("RUSTC_BOOTSTRAP", "1");
    if let Some(pkg) = &t.pkg {
        for p in pkg.split_whitespace() {
            cargo.arg("-p").arg(p);
        }
    }
    if !cargo.status().map(|s| s.success()).unwrap_or(false) {
        println!("  {} {} — build failed", red("✗"), bold(&t.name));
        return false;
    }
    let target = root.join(&t.dir).join("target/release").join(t.bin());
    if !link_bin(&target, &t.name) {
        return false;
    }
    if let Some(alias) = &t.alias {
        if !link_bin(&target, alias) {
            return false;
        }
    }
    true
}

fn cmd_install(names: &[String]) {
    let all = names.is_empty() || names.iter().any(|n| n == "--all" || n == "all");
    let targets: Vec<&Tool> = if all {
        registry().iter().collect()
    } else {
        names
            .iter()
            .filter_map(|n| {
                tool(n).or_else(|| {
                    println!("  {}: not a registered tool (see `cyber tools`)", red(n));
                    None
                })
            })
            .collect()
    };
    if targets.is_empty() {
        return;
    }
    let root = cyber_root();
    let _ = std::fs::create_dir_all(bin_dir());
    let (mut ok, mut fail) = (0u32, 0u32);
    for t in targets {
        if !root.join(t.repo()).exists() && !sync_one(t.repo(), &root) {
            fail += 1;
            continue;
        }
        if install_one(t, &root) {
            ok += 1;
        } else {
            fail += 1;
        }
    }
    let tail = if fail > 0 {
        red(&format!(", {fail} failed"))
    } else {
        String::new()
    };
    println!("  {} {ok} installed{tail}", green("✓"));
}

fn show_tools() {
    println!(
        "{}",
        dim("toolchain — advanced · day-one is `cyber sync` → spacepussy-test")
    );
    for t in registry() {
        let link = bin_dir().join(&t.name);
        let (mark, note) = if std::fs::metadata(&link).is_ok() {
            (green("●"), "")
        } else if std::fs::symlink_metadata(&link).is_ok() {
            (yellow("⚠"), " — stale, run `cyber install`")
        } else {
            (dim("○"), "")
        };
        let alias = t
            .alias
            .as_deref()
            .map(|a| format!("  {}", dim(&format!("({a})"))))
            .unwrap_or_default();
        println!(
            "  {} {}  {}{}{}",
            mark,
            bold(&format!("{:<10}", t.name)),
            dim(&t.desc),
            alias,
            dim(note)
        );
    }
}

fn dispatch_tool(name: &str, args: &[String]) {
    match Command::new(name).args(args).status() {
        Ok(s) => std::process::exit(s.code().unwrap_or(0)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "  {} not installed — {}",
                bold(name),
                green(&format!("cyber install {name}"))
            );
            std::process::exit(2);
        }
        Err(e) => {
            println!("  {}: {name}: {e}", red("error"));
            std::process::exit(1);
        }
    }
}

// ── help / status ───────────────────────────────────────────────────────────

fn show_help() {
    let n = Network::DEFAULT;
    println!(
        "{} {} — spacepussy-test CLI",
        bold("cyber"),
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  default: {}  ({})", n.chain_id(), n.rpc());
    println!("  {}", dim(n.role()));
    println!();
    println!("  {}", bold("day one"));
    println!(
        "  {}          probe spacepussy-test (local soft3 node)",
        bold("cyber sync")
    );
    println!(
        "  {}       print endpoints",
        bold("cyber network")
    );
    println!("  {}         soft3 manifesto", bold("cyber manifesto"));
    println!();
    println!("  {}", bold("not product networks"));
    println!(
        "  {}",
        dim("space-pussy · bostrom  — cosmos bootloader on cybernode (migration sources)")
    );
    println!();
    println!("  {}", bold("faces (optional)"));
    println!("  {} …       forward if installed", bold("cyber soft3"));
    println!("  {} …           forward if installed", bold("cyber cy"));
    println!();
    println!("  {}", bold("toolchain (advanced)"));
    println!("  {}          list stack tools", bold("cyber tools"));
    println!(
        "  {}         clone sources into $CYBER_ROOT",
        bold("cyber source")
    );
    println!(
        "  {}  build tools from source onto PATH",
        bold("cyber install --all")
    );
    println!();
    println!(
        "  {}",
        dim("crate: true-cyber · binary: cyber · https://cyber.page")
    );
    println!("  {}", dim("don't trust. don't fear. don't beg."));
}

fn show_status(net: Network) {
    println!(
        "{} {}  ·  {}",
        bold("cyber"),
        env!("CARGO_PKG_VERSION"),
        net.chain_id()
    );
    println!("  {}", dim(net.role()));
    println!("  rpc  {}", net.rpc());
    println!();
    println!("  {}  {}", dim("probe:"), green("cyber sync"));
    println!("  {}  {}", dim("help:"), green("cyber help"));
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let (net, args) = parse_global_network(&raw);

    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
    let rest: &[String] = if args.is_empty() { &[] } else { &args[1..] };

    match cmd {
        "" | "status" => show_status(net),
        "sync" => cmd_sync(net),
        "network" | "net" => {
            let n = rest
                .first()
                .map(|s| parse_net(s))
                .unwrap_or(net);
            cmd_network(n);
        }
        "manifesto" => cmd_manifesto(),
        "soft3" => forward(
            "soft3",
            rest,
            "optional: cargo install soft3",
        ),
        "cy" | "cyb" => forward(
            "cy",
            rest,
            "optional: cargo install cyb",
        ),
        "source" | "clone" => cmd_source(rest),
        "install" => cmd_install(rest),
        "tools" | "deps" => show_tools(),
        "help" | "--help" | "-h" | "?" => show_help(),
        "version" | "--version" | "-V" => {
            println!(
                "cyber {} (true-cyber) · soft3 {} · {}",
                env!("CARGO_PKG_VERSION"),
                soft3::VERSION,
                Network::DEFAULT.chain_id()
            );
        }
        name if tool(name).is_some() => dispatch_tool(name, rest),
        name => {
            eprintln!(
                "  {} {} — see {}",
                red("unknown:"),
                bold(name),
                green("cyber help")
            );
            std::process::exit(1);
        }
    }
}
