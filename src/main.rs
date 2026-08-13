// ---
// tags: cyber, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cyber — the bootstrap face of the cyber stack.
//!
//! `cargo install true-cyber` puts this binary on PATH as `cyber`. From there:
//!
//! ```text
//! cyber                  status of the stack — repos and tools
//! cyber sync             clone the missing source repos into $CYBER_ROOT
//! cyber install --all    build every tool from source and link it onto PATH
//! cyber <tool> …         run a tool by name (hemera, nox, rune, zheng, …)
//! cyber graph            build optica and serve the knowledge graph locally
//! ```
//!
//! The registry (tools.toml, embedded) is the single source of truth: it lists
//! each tool, the repo that builds it, and the binary it produces. The canonical
//! copy lives at cyb/cli/tools.toml — keep the vendored copy in sync.

use std::path::{Path, PathBuf};
use std::process::Command;

const ORG: &str = "https://github.com/cyberia-to";

// ── registry ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct Registry {
    tool: Vec<Tool>,
}

/// One entry in the toolset registry.
#[derive(serde::Deserialize)]
struct Tool {
    /// the command name — and the symlink placed on PATH
    name: String,
    /// build directory under `$CYBER_ROOT`
    dir: String,
    /// cargo package(s) for `-p` (a single-crate directory omits it)
    pkg: Option<String>,
    /// built binary name; defaults to `name` when they match
    bin: Option<String>,
    /// short name; also resolves and is linked onto PATH (e.g. `cg` → cybergraph)
    alias: Option<String>,
    /// one-line description
    desc: String,
}

impl Tool {
    /// The built binary's file name.
    fn bin(&self) -> &str {
        self.bin.as_deref().unwrap_or(&self.name)
    }

    /// The repo that holds this tool — the first component of its build dir.
    fn repo(&self) -> &str {
        self.dir.split('/').next().unwrap_or(&self.dir)
    }
}

/// The registry, embedded at build time and parsed once.
fn registry() -> &'static [Tool] {
    static REG: std::sync::OnceLock<Vec<Tool>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        toml::from_str::<Registry>(include_str!("../tools.toml"))
            .expect("tools.toml is malformed")
            .tool
    })
}

/// The registered tool of this name, if any. This is the dispatch boundary —
/// only names in the registry run, so `cyber ls` is unknown, not `/bin/ls`.
fn tool(name: &str) -> Option<&'static Tool> {
    registry()
        .iter()
        .find(|t| t.name == name || t.alias.as_deref() == Some(name))
}

/// Every repo the stack builds from: registry repos plus the knowledge graph
/// and its publisher.
fn repos() -> Vec<&'static str> {
    let mut rs: Vec<&str> = registry().iter().map(|t| t.repo()).collect();
    rs.push("cyber");
    rs.push("optica");
    rs.sort();
    rs.dedup();
    rs
}

// ── paths ───────────────────────────────────────────────────────────────────

/// The cyber source root that holds every tool's repo (`$CYBER_ROOT` or `~/cyber`).
fn cyber_root() -> PathBuf {
    if let Some(r) = std::env::var_os("CYBER_ROOT") {
        return PathBuf::from(r);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Path::new(&home).join("cyber")
}

/// Where installed tools are symlinked (on PATH).
fn bin_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Path::new(&home).join(".cargo").join("bin")
}

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

// ── sync ────────────────────────────────────────────────────────────────────

/// Clone a repo into `$CYBER_ROOT/<name>` if missing. Present repos are left
/// untouched — local state belongs to the neuron, not the installer.
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

/// `cyber sync [name…]` — clone missing source repos.
fn cmd_sync(names: &[String]) {
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

// ── install ─────────────────────────────────────────────────────────────────

/// `cyber install [name…|--all]` — build tools from the registry and link them
/// onto PATH. This is the whole install mechanism; there is no external script.
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

/// Build one tool in release and symlink it onto PATH. Cargo output streams so
/// the build is visible. `$CYBER_ROOT/<dir>` is the workspace; the binary lands
/// at `<dir>/target/release/<bin>`.
fn install_one(t: &Tool, root: &Path) -> bool {
    println!("  {} {}", dim("building"), bold(&t.name));
    let mut cargo = Command::new("cargo");
    cargo
        .arg("build")
        .arg("--release")
        .current_dir(root.join(&t.dir))
        .env("RUSTC_BOOTSTRAP", "1"); // the workspace pulls crates needing nightly features
    // `pkg` may name several packages (space-separated) — a group like strata
    // builds its dispatcher and every algebra CLI in one invocation.
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
    // link the name, and its short alias (if any), onto PATH — both point at the
    // one binary, so `cyber neural` / `cyber neu` and the standalone commands work.
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

/// Symlink `<bin_dir>/<name>` → `target`, replacing any prior link. Reports the link.
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

// ── tools ───────────────────────────────────────────────────────────────────

/// List the toolset from the registry, each with its install status.
fn show_tools() {
    println!(
        "{}",
        dim("the cyber toolset — `cyber <name> …` runs one · `cyber install --all` builds them")
    );
    for t in registry() {
        let link = bin_dir().join(&t.name);
        let (mark, note) = if std::fs::metadata(&link).is_ok() {
            (green("●"), "") // link resolves to a real binary
        } else if std::fs::symlink_metadata(&link).is_ok() {
            (yellow("⚠"), " — stale, run `cyber install`") // link exists, target gone
        } else {
            (dim("○"), "") // not installed
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

/// Status of the stack: the source root, its repos, and the toolset.
fn show_status() {
    let root = cyber_root();
    println!("{}  {}", bold("cyber"), dim(&root.display().to_string()));
    let (mut here, mut gone) = (0u32, 0u32);
    for r in repos() {
        if root.join(r).exists() {
            here += 1;
        } else {
            gone += 1;
        }
    }
    let repos_note = if gone > 0 {
        format!("{here} repos present, {gone} missing — `cyber sync` clones them")
    } else {
        format!("{here} repos present")
    };
    println!("  {}", dim(&repos_note));
    println!();
    show_tools();
}

// ── graph ───────────────────────────────────────────────────────────────────

/// `cyber graph [args…]` — the knowledge graph, locally. Ensures the cyber and
/// optica repos are present, builds optica once, and serves the root graph.
/// Extra args pass through to `optica serve`.
fn cmd_graph(args: &[String]) {
    let root = cyber_root();
    let _ = std::fs::create_dir_all(&root);
    for r in ["cyber", "optica"] {
        if !root.join(r).exists() && !sync_one(r, &root) {
            return;
        }
    }
    let optica = root.join("optica/target/release/optica");
    if !optica.exists() {
        println!("  {} {}", dim("building"), bold("optica"));
        let ok = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(root.join("optica"))
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            println!("  {} optica — build failed", red("✗"));
            return;
        }
    }
    let graph = root.join("cyber");
    println!(
        "  {} {}",
        dim("serving"),
        dim(&graph.display().to_string())
    );
    let _ = Command::new(&optica)
        .arg("serve")
        .arg(&graph)
        .args(args)
        .status();
}

// ── dispatch ────────────────────────────────────────────────────────────────

/// Run a registered tool, passing argv verbatim.
fn dispatch(name: &str, args: &[String]) {
    match Command::new(name).args(args).status() {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "  {} not installed — {}",
                bold(name),
                green(&format!("cyber install {name}"))
            );
        }
        Err(e) => println!("  {}: {name}: {e}", red("error")),
    }
}

// ── help ────────────────────────────────────────────────────────────────────

fn show_help() {
    println!("{} — the cyber stack, from source to PATH", bold("cyber"));
    println!();
    println!("  {}          status — repos and tools", bold("cyber"));
    println!("  {}    clone the missing source repos", bold("cyber sync"));
    println!(
        "  {} build every tool and link it onto PATH",
        bold("cyber install --all")
    );
    println!(
        "  {} build one tool (hemera, nox, rune, zheng, …)",
        bold("cyber install <name>")
    );
    println!(
        "  {}  run a tool by name — `cyber tools` lists them",
        bold("cyber <tool> …")
    );
    println!(
        "  {}   serve the knowledge graph locally",
        bold("cyber graph")
    );
    println!();
    println!(
        "  {}",
        dim("source root: $CYBER_ROOT (default ~/cyber) · links: ~/.cargo/bin")
    );
    println!("  {}", dim("don't trust. don't fear. don't beg."));
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let rest = if args.is_empty() { &[] } else { &args[1..] };
    match cmd {
        "" | "status" => show_status(),
        "tools" | "deps" => show_tools(),
        "sync" => cmd_sync(rest),
        "install" => cmd_install(rest),
        "graph" => cmd_graph(rest),
        "help" | "--help" | "-h" | "?" => show_help(),
        "version" | "--version" | "-V" => {
            println!("cyber {} (true-cyber)", env!("CARGO_PKG_VERSION"))
        }
        name if tool(name).is_some() => dispatch(name, rest),
        name => {
            println!(
                "  {} {} — see {}",
                red("unknown:"),
                bold(name),
                green("cyber help")
            );
            std::process::exit(1);
        }
    }
}
