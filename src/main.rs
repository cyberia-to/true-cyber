// ---
// tags: cyber, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cyber — product CLI for spacepussy-test on cybernode.
//!
//! ```text
//! cargo install true-cyber
//! cyber sync                 # probe public spacepussy-test
//! ```

use soft3::network::{self, Network};
use std::path::{Path, PathBuf};
use std::process::Command;

const ORG: &str = "https://github.com/cyberia-to";

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
        eprintln!("`{name}` is a cosmos bootloader chain on cybernode — not spacepussy-test.");
        eprintln!(
            "product network: {} · {}",
            Network::DEFAULT.chain_id(),
            Network::DEFAULT.rpc()
        );
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
            if !s.chain_id.is_empty() {
                println!("  chain_id         {}", s.chain_id);
            }
            if !s.moniker.is_empty() {
                println!("  moniker          {}", s.moniker);
            }
            if s.latest_height > 0 || s.reachable {
                println!("  latest_height    {}", s.latest_height);
            }
            if s.earliest_height > 0 {
                println!("  earliest_height  {}", s.earliest_height);
            }
            println!(
                "  catching_up      {}",
                if s.catching_up { "yes" } else { "no" }
            );
        }
        Err(e) => {
            println!("  reachable        {}", red("no"));
            println!("  detail           {e}");
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
    if net == Network::DEFAULT {
        println!("  (product default)");
    }
}

fn cmd_manifesto() {
    for line in soft3::manifesto() {
        println!("{line}");
    }
}

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
            std::process::exit(2);
        }
    }
}

// toolchain bootstrap (advanced) — kept compact

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
fn cyber_root() -> PathBuf {
    if let Some(r) = std::env::var_os("CYBER_ROOT") {
        return PathBuf::from(r);
    }
    Path::new(&std::env::var("HOME").unwrap_or_else(|_| ".".into())).join("cyber")
}
fn bin_dir() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".cargo")
        .join("bin")
}
fn sync_one(name: &str, root: &Path) -> bool {
    let target = root.join(name);
    if target.exists() {
        println!("  {} {}  {}", green("●"), bold(name), dim("present"));
        return true;
    }
    let url = format!("{ORG}/{name}.git");
    println!("  {} {}  {}", dim("…"), bold(name), dim(&url));
    Command::new("git")
        .args(["clone", "--depth", "1", &url])
        .arg(&target)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
fn cmd_source(names: &[String]) {
    let root = cyber_root();
    let _ = std::fs::create_dir_all(&root);
    let targets: Vec<&str> = if names.is_empty() {
        let mut rs: Vec<&str> = registry().iter().map(|t| t.repo()).collect();
        rs.extend(["cyber", "optica", "soft3", "true-cyber"]);
        rs.sort();
        rs.dedup();
        rs
    } else {
        names.iter().map(|s| s.as_str()).collect()
    };
    for name in targets {
        let _ = sync_one(name, &root);
    }
}
fn show_tools() {
    for t in registry() {
        let link = bin_dir().join(&t.name);
        let mark = if std::fs::metadata(&link).is_ok() {
            green("●")
        } else {
            dim("○")
        };
        println!(
            "  {} {}  {}",
            mark,
            bold(&format!("{:<10}", t.name)),
            dim(&t.desc)
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
fn cmd_install(names: &[String]) {
    let all = names.is_empty() || names.iter().any(|n| n == "--all" || n == "all");
    let targets: Vec<&Tool> = if all {
        registry().iter().collect()
    } else {
        names.iter().filter_map(|n| tool(n)).collect()
    };
    let root = cyber_root();
    for t in targets {
        if !root.join(t.repo()).exists() && !sync_one(t.repo(), &root) {
            continue;
        }
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
        let _ = cargo.status();
    }
}

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
    println!(
        "  {}          probe public spacepussy-test",
        bold("cyber sync")
    );
    println!("  {}       print endpoints", bold("cyber network"));
    println!("  {}         manifesto", bold("cyber manifesto"));
    println!();
    println!(
        "  {}",
        dim("not product: cosmos space-pussy / bostrom (bootloader)")
    );
    println!();
    println!(
        "  {}",
        dim("crate: true-cyber · https://cyber.page/install")
    );
}

fn show_status(net: Network) {
    println!(
        "{} {}  ·  {}",
        bold("cyber"),
        env!("CARGO_PKG_VERSION"),
        net.chain_id()
    );
    println!("  rpc  {}", net.rpc());
    println!("  {}  {}", dim("probe:"), green("cyber sync"));
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let (net, args) = parse_global_network(&raw);
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
    let rest: &[String] = if args.is_empty() { &[] } else { &args[1..] };

    match cmd {
        "" | "status" => show_status(net),
        "sync" => cmd_sync(net),
        "network" | "net" => {
            let n = rest.first().map(|s| parse_net(s)).unwrap_or(net);
            cmd_network(n);
        }
        "manifesto" => cmd_manifesto(),
        "soft3" => forward("soft3", rest, "optional: cargo install soft3"),
        "cy" | "cyb" => forward("cy", rest, "optional: cargo install cyb"),
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
