//! Headless native graph client using the common robot and graph owners.
mod http;
mod legacy;
use cyb_core::{
    GraphSession,
    native_mirror::Mirror,
    robot::{
        self, NetworkRef, Registry, custody,
        host::{Host, lookup, nonce},
        remote::{hex, parse_id},
    },
};
use cybergraph::Particle;
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
type Result<T> = std::result::Result<T, String>;
const DEFAULT_RPC: &str = "https://cyb.ai/spacepussy-test";
const CHAIN: &str = "spacepussy-test";
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

struct Options {
    words: Vec<String>,
    flags: BTreeMap<String, String>,
}
impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter();
        let mut words = Vec::new();
        let mut flags = BTreeMap::new();
        while let Some(arg) = args.next() {
            if arg.starts_with("--") && !matches!(arg.as_str(), "--help" | "--version") {
                if !matches!(
                    arg.as_str(),
                    "--home"
                        | "--rpc"
                        | "--network"
                        | "--attachment"
                        | "--request"
                        | "--amount"
                        | "--valence"
                ) {
                    return Err(format!(
                        "unsupported option {arg}; --neuron labels cannot authorize a signal"
                    ));
                }
                let value = args
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                if value.starts_with("--") || value.is_empty() {
                    return Err(format!("missing value for {arg}"));
                }
                if flags.insert(arg.clone(), value).is_some() {
                    return Err(format!("duplicate option {arg}"));
                }
            } else {
                words.push(arg);
            }
        }
        Ok(Self { words, flags })
    }
    fn value(&self, key: &str) -> Option<&str> {
        self.flags.get(key).map(String::as_str)
    }
    fn allow(&self, allowed: &[&str]) -> Result<()> {
        for flag in self.flags.keys() {
            if flag != "--home" && !allowed.contains(&flag.as_str()) {
                return Err(format!("{flag} does not apply to this command"));
            }
        }
        Ok(())
    }
    fn network(&self) -> Result<Particle> {
        parse_id(
            self.value("--network")
                .ok_or("an explicit full --network ID is required")?,
        )
    }
    fn endpoint(&self) -> Result<http::Http> {
        http::Http::new(self.value("--rpc").unwrap_or(DEFAULT_RPC), self.network()?)
    }
    fn home(&self) -> Result<PathBuf> {
        if let Some(home) = self.value("--home") {
            return Ok(home.into());
        }
        if let Some(home) = std::env::var_os("CYBER_HOME") {
            return Ok(home.into());
        }
        Ok(
            PathBuf::from(std::env::var_os("HOME").ok_or("HOME or CYBER_HOME is required")?)
                .join(".cyber")
                .join(CHAIN),
        )
    }
}
struct Client {
    session: GraphSession,
    registry: Registry,
    host: Host,
}
impl Client {
    fn open(home: &Path) -> Result<Self> {
        let session = GraphSession::open(home.join("log")).map_err(error)?;
        let registry = Registry::from_database(session.database())?;
        let host = Host::new(
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.into()),
            home.into(),
            "true-cyber".into(),
        )?;
        for warning in host.initialize(&registry)?.warnings {
            eprintln!("{warning}");
        }
        Ok(Self {
            session,
            registry,
            host,
        })
    }
    fn sync(&mut self, transport: &mut http::Http) -> Result<usize> {
        let mirror = Mirror::new(transport.network);
        let mut count = 0;
        for _ in 0..4096 {
            let page = transport.history(mirror.cursor(&self.session)?)?;
            let progress = mirror.ingest(&mut self.session, &page)?;
            count += progress.observations;
            if progress.observations == 0 {
                return Ok(count);
            }
        }
        Err(format!(
            "sync reached 4096 pages; {count} observations persisted; run sync again"
        ))
    }
    fn attachment(&self, options: &Options) -> Result<robot::Attachment> {
        let id = lookup(
            &self.registry,
            options.value("--attachment").unwrap_or("selected"),
        )?;
        let a = self
            .registry
            .robot()?
            .attachment(id)
            .cloned()
            .ok_or("attachment missing")?;
        if a.binding.network != NetworkRef::Native(options.network()?) {
            return Err("attachment network differs from explicit destination".into());
        }
        a.binding
            .subject
            .native()
            .ok_or("native link requires a native subject")?;
        Ok(a)
    }
    fn relay(&self, attachment: Particle, transport: &mut http::Http) -> Result<usize> {
        let a = self
            .registry
            .robot()?
            .attachment(attachment)
            .cloned()
            .ok_or("attachment missing")?;
        if a.binding.network != NetworkRef::Native(transport.network) {
            return Err("outbox network mismatch".into());
        }
        // Missing custody may still permit read-only lookup of an accepted action.
        let key = a
            .key_ref
            .as_deref()
            .and_then(|p| custody::load(Path::new(p)).ok());
        let mut count = 0;
        for (position, action) in self.registry.native_page(attachment)? {
            if let Some(envelope) = action {
                self.registry.deliver_native(
                    envelope.action.request,
                    key.as_ref().map(|k| k.handle.as_ref()),
                    transport.network,
                    transport,
                )?;
                if envelope.action.kind == "native/signal" {
                    self.registry
                        .reconcile_signal(&self.session, envelope.action.request)?;
                }
                count += 1;
            }
            self.registry.advance_native_cursor(attachment, position)?;
        }
        Ok(count)
    }
    fn link(&mut self, options: &Options, from: &str, to: &str) -> Result<()> {
        let a = self.attachment(options)?;
        let subject = a
            .binding
            .subject
            .native()
            .ok_or("native subject required")?;
        let from = particle(from)?;
        let to = particle(to)?;
        let amount = options
            .value("--amount")
            .unwrap_or("1")
            .parse::<u64>()
            .map_err(error)?;
        if amount == 0 {
            return Err("amount must be positive".into());
        }
        let valence = options
            .value("--valence")
            .unwrap_or("0")
            .parse::<i8>()
            .map_err(error)?;
        let request = match options.value("--request") {
            Some(id) => parse_id(id)?,
            None => nonce()?,
        };
        println!("request {}", hex(&request));
        use std::io::Write;
        std::io::stdout().flush().map_err(error)?;
        let mut transport = options.endpoint()?;
        // Establish the pinned profile before any new local admission.
        transport.verify()?;
        let original = self.registry.admitted(request)?;
        if let Some(action) = &original {
            let signal = foculus::signal_codec::decode_signal(&action.payload).map_err(error)?;
            if action.kind != "native/signal"
                || action.attachment != a.id
                || action.context.subject.native() != Some(subject)
                || action.context.network != NetworkRef::Native(transport.network)
                || signal.neuron != subject
                || signal.network != transport.network
                || signal.links.len() != 1
                || !signal.delta_pi.is_empty()
                || !signal.box_moves.is_empty()
                || signal.proof.is_some()
                || {
                    let l = &signal.links[0];
                    (l.neuron, l.from, l.to, l.token, l.amount, l.valence)
                        != (subject, from, to, [0; 32], amount, valence)
                }
            {
                return Err("request conflicts with its original link/scope".into());
            }
        }
        if self.registry.attempt(request)?.is_none() {
            self.sync(&mut transport)?;
            // New independent sends cannot leap over a pending earlier sequence.
            let mut exhausted = true;
            for _ in 0..4096 {
                let page = self.registry.native_page(a.id)?;
                if page.is_empty() {
                    exhausted = false;
                    break;
                }
                for (position, action) in page {
                    if let Some(action) = action {
                        if self
                            .registry
                            .native_receipt(action.action.request)?
                            .is_none()
                        {
                            return Err(format!(
                                "resolve prior request {} with relay before a new link",
                                hex(&action.action.request)
                            ));
                        }
                    }
                    self.registry.advance_native_cursor(a.id, position)?;
                }
            }
            if exhausted {
                return Err("outbox scan reached page limit; relay and retry".into());
            }
            let key = custody::load(Path::new(
                a.key_ref.as_deref().ok_or("attachment has no custody")?,
            ))?;
            let signal = if let Some(action) = original {
                foculus::signal_codec::decode_signal(&action.payload).map_err(error)?
            } else {
                let mut signal =
                    self.session
                        .prepare_signal(subject, transport.network, [(from, to, amount)]);
                signal.links[0].valence = valence;
                signal
            };
            self.registry.publish_signal(
                &mut self.session,
                &key.handle,
                a.id,
                a.binding.revision,
                request,
                signal,
            )?;
        }
        self.registry.reconcile_signal(&self.session, request)?;
        let key = a
            .key_ref
            .as_deref()
            .and_then(|p| custody::load(Path::new(p)).ok());
        let receipt = self.registry.deliver_native(
            request,
            key.as_ref().map(|k| k.handle.as_ref()),
            transport.network,
            &mut transport,
        )?;
        self.sync(&mut transport)?;
        self.registry.reconcile_signal(&self.session, request)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&receipt.value).map_err(error)?
        );
        Ok(())
    }
    fn status(&self, home: &Path) -> Result<()> {
        let graph = self.session.graph();
        println!("{}",serde_json::to_string_pretty(&json!({
            "schema":"true-cyber/status/1", "home":home,"profile":"local graph projection",
            "height":self.session.node().height(),"local_root":hex(&graph.bbg.state.root()),
            "neurons":graph.chains.len(),"signals":graph.chains.values().map(|c|c.entries.len()).sum::<usize>(),
            "particles":graph.bbg.state.particles.len(),"axons":graph.bbg.state.axons_out.len(),
            "attachments":self.registry.robot()?.attachments.len(),"legacy":legacy::inspect(home)?,
            "consensus_verified":false
        })).map_err(error)?);
        Ok(())
    }
}
fn particle(text: &str) -> Result<Particle> {
    if text.is_empty() || text.len() > 65536 {
        return Err("particle label must contain 1..65536 bytes".into());
    }
    let candidate = text.strip_prefix("0x").unwrap_or(text);
    if !candidate.is_empty()
        && candidate.len() <= 64
        && candidate.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return parse_id(&format!("{candidate:0>64}"));
    }
    Ok(*hemera::hash(text.as_bytes()).as_bytes())
}
fn run(options: Options) -> Result<()> {
    let words: Vec<_> = options.words.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["help" | "--help" | "-h" | "?"] => {
            options.allow(&[])?;
            println!("{}", include_str!("help.txt"));
            return Ok(());
        }
        ["version" | "--version" | "-V"] => {
            options.allow(&[])?;
            println!("cyber {} (true-cyber)", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        ["network" | "net"] => {
            options.allow(&[])?;
            println!(
                "default transport {DEFAULT_RPC}\nnative profile {}\nnetwork identity requires explicit --network HEX; obtain it from your trusted genesis",
                robot::remote::PROFILE
            );
            return Ok(());
        }
        _ => {}
    }
    // Validate command/flags before opening (and potentially importing) storage.
    match words.as_slice() {
        [] | ["status"] | ["neuron", ..] | ["legacy", "inspect"] | ["legacy", "export", _] => {
            options.allow(&[])?
        }
        ["sync"] => options.allow(&["--network", "--rpc"])?,
        ["relay"] => options.allow(&["--network", "--rpc", "--attachment"])?,
        ["receipt", _] => options.allow(&["--network", "--rpc"])?,
        ["link", _, _] => options.allow(&[
            "--network",
            "--rpc",
            "--attachment",
            "--request",
            "--amount",
            "--valence",
        ])?,
        _ => return Err("unknown command or missing arguments; cyber help".into()),
    }
    let home = options.home()?;
    if words.first() == Some(&"legacy") {
        match words.as_slice() {
            [_, "inspect"] => println!(
                "{}",
                serde_json::to_string_pretty(&legacy::inspect(&home)?).map_err(error)?
            ),
            [_, "export", path] => legacy::export(&home, Path::new(path))?,
            _ => unreachable!(),
        }
        return Ok(());
    }
    if matches!(
        words.first().copied(),
        Some("sync" | "relay" | "receipt" | "link")
    ) {
        options.network()?;
    }
    let mut client = Client::open(&home)?;
    match words.as_slice() {
        [] | ["status"] => client.status(&home),
        ["neuron", tail @ ..] => {
            println!("{}", client.host.execute(&client.registry, tail)?);
            Ok(())
        }
        ["sync"] => {
            let mut transport = options.endpoint()?;
            println!(
                "{} endpoint observations retained; local projection, not consensus finality",
                client.sync(&mut transport)?
            );
            Ok(())
        }
        ["relay"] => {
            let a = client.attachment(&options)?;
            let mut transport = options.endpoint()?;
            println!(
                "{} original requests accepted",
                client.relay(a.id, &mut transport)?
            );
            client.sync(&mut transport)?;
            Ok(())
        }
        ["receipt", request] => {
            let request = parse_id(request)?;
            let mut transport = options.endpoint()?;
            let receipt =
                client
                    .registry
                    .deliver_native(request, None, transport.network, &mut transport)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&receipt.value).map_err(error)?
            );
            Ok(())
        }
        ["link", from, to] => client.link(&options, from, to),
        _ => unreachable!(),
    }
}
fn main() {
    if let Err(e) = Options::parse(std::env::args().skip(1)).and_then(run) {
        eprintln!("cyber: {e}");
        std::process::exit(1);
    }
}
