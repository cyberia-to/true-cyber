use cyb_core::{
    GraphSession,
    robot::{Registry, custody, host::Host, remote::hex},
};
use serde_json::{Value, json};
use soft3::node::Node;
use std::{
    net::{Shutdown, TcpListener, TcpStream},
    path::Path,
    process::{Command, Output},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
struct Peer {
    url: String,
    network: [u8; 32],
    node: Arc<Mutex<Node>>,
    stop: Arc<AtomicBool>,
    lose: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    _dir: tempfile::TempDir,
}
impl Peer {
    fn new(time: u64) -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("genesis.json"),serde_json::to_vec(&json!({"chain_id":"spacepussy-test","genesis_time":time,"engine":"cybergraph+bbg","protocol":"soft3/spacepussy-test/v1"})).unwrap()).unwrap();
        let node = Node::open(dir.path().into(), "true-cyber-test".into()).unwrap();
        let network = node.enable_authentication().unwrap();
        let node = Arc::new(Mutex::new(node));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let lose = Arc::new(AtomicBool::new(false));
        let (n, s, l) = (node.clone(), stop.clone(), lose.clone());
        let thread = thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                if s.load(Ordering::SeqCst) {
                    break;
                }
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut first = [0; 5];
                let count = stream.peek(&mut first).unwrap();
                if count >= 4 && &first[..4] == b"POST" && l.swap(false, Ordering::SeqCst) {
                    // The request is still read and durably accepted; its response
                    // cannot reach the client. The real HTTP server handles it.
                    stream.shutdown(Shutdown::Write).unwrap();
                }
                let _ = soft3::node::serve_connection(stream, &n);
            }
        });
        Self {
            url: format!("http://{addr}"),
            network,
            node,
            stop,
            lose,
            thread: Some(thread),
            _dir: dir,
        }
    }
    fn count(&self) -> u64 {
        self.node.lock().unwrap().signal_count()
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.url.strip_prefix("http://").unwrap());
        self.thread.take().unwrap().join().unwrap();
    }
}
fn call(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cyber"))
        .env("CYBER_HOME", home)
        .env("HOME", home)
        .args(args)
        .output()
        .unwrap()
}
fn ok(home: &Path, args: &[&str]) -> String {
    let out = call(home, args);
    assert!(
        out.status.success(),
        "{args:?}: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
fn bad(home: &Path, args: &[&str]) -> String {
    let out = call(home, args);
    assert!(!out.status.success(), "unexpected success: {args:?}");
    String::from_utf8(out.stderr).unwrap()
}
fn attachment(home: &Path) -> [u8; 32] {
    let session = GraphSession::open(home.join("log")).unwrap();
    Registry::from_database(session.database())
        .unwrap()
        .robot()
        .unwrap()
        .selected
        .unwrap()
}
fn count(home: &Path) -> usize {
    let value: Value = serde_json::from_str(&ok(home, &["status"])).unwrap();
    value["signals"].as_u64().unwrap() as usize
}
#[test]
fn real_processes_sign_retry_preserve_network_and_revoke_without_new_keys() {
    let peer = Peer::new(101);
    let other = Peer::new(102);
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    let a = hex(&peer.network);
    let b = hex(&other.network);
    let request = hex(&[10; 32]);
    assert_eq!(count(home), 0);
    assert!(!home.join("keys").exists());
    ok(home, &["neuron", "create", "a", &a]);
    let attachment = hex(&attachment(home));
    let custody = std::fs::read(home.join("keys/a.mnemonic")).unwrap();
    let link = [
        "link",
        "alpha",
        "beta",
        "--network",
        &a,
        "--rpc",
        &peer.url,
        "--request",
        &request,
        "--amount",
        "7",
        "--valence",
        "-3",
    ];
    ok(home, &link);
    assert_eq!(peer.count(), 1);
    assert_eq!(count(home), 1);
    ok(home, &link);
    assert_eq!(peer.count(), 1);
    assert_eq!(count(home), 1);
    let mut changed = link;
    changed[2] = "gamma";
    assert!(bad(home, &changed).contains("conflicts"));
    let wrong = [
        "link",
        "a",
        "b",
        "--network",
        &a,
        "--rpc",
        &other.url,
        "--request",
        &hex(&[11; 32]),
    ];
    assert!(bad(home, &wrong).contains("pinned"));
    assert_eq!(peer.count(), 1);
    assert_eq!(other.count(), 0);
    ok(home, &["neuron", "create", "b", &b]);
    assert!(bad(home, &link).contains("network differs"));
    ok(
        home,
        &[
            "link",
            "delta",
            "omega",
            "--network",
            &b,
            "--rpc",
            &other.url,
        ],
    );
    assert_eq!(other.count(), 1);
    assert_eq!(count(home), 2);
    ok(home, &["neuron", "revoke", &attachment]);
    ok(
        home,
        &["receipt", &request, "--network", &a, "--rpc", &peer.url],
    );
    assert!(
        bad(
            home,
            &[
                "link",
                "x",
                "y",
                "--attachment",
                &attachment,
                "--network",
                &a,
                "--rpc",
                &peer.url
            ]
        )
        .contains("grant")
    );
    assert_eq!(peer.count(), 1);
    assert_eq!(
        std::fs::read(home.join("keys/a.mnemonic")).unwrap(),
        custody
    );
    let mut session = GraphSession::open(home.join("log")).unwrap();
    let registry = Registry::from_database(session.database()).unwrap();
    let action = registry.attempt([10; 32]).unwrap().unwrap();
    let signal = foculus::signal_codec::decode_signal(&action.payload).unwrap();
    assert_eq!((signal.links[0].amount, signal.links[0].valence), (7, -3));
    assert_eq!(signal.network, peer.network);
    // The shared prepared-signal API refuses forged subject/network before admission.
    let selected = registry.robot().unwrap().selected().unwrap().clone();
    let key =
        cyb_core::robot::custody::load(Path::new(selected.key_ref.as_deref().unwrap())).unwrap();
    let mut forged = session.prepare_signal(key.handle.subject(), peer.network, []);
    assert!(
        registry
            .publish_signal(
                &mut session,
                &key.handle,
                selected.id,
                selected.binding.revision,
                [80; 32],
                forged.clone()
            )
            .is_err()
    );
    forged.network = other.network;
    forged.neuron = [0; 32];
    assert!(
        registry
            .publish_signal(
                &mut session,
                &key.handle,
                selected.id,
                selected.binding.revision,
                [80; 32],
                forged
            )
            .is_err()
    );
    assert!(registry.admitted([80; 32]).unwrap().is_none());
}
#[test]
fn lost_http_response_reopens_and_resolves_exact_original_after_revocation() {
    let peer = Peer::new(103);
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    let network = hex(&peer.network);
    let request = hex(&[12; 32]);
    ok(home, &["neuron", "create", "a", &network]);
    let id = hex(&attachment(home));
    peer.lose.store(true, Ordering::SeqCst);
    let link = [
        "link",
        "alpha",
        "beta",
        "--network",
        &network,
        "--rpc",
        &peer.url,
        "--request",
        &request,
    ];
    bad(home, &link);
    assert_eq!(peer.count(), 1);
    assert_eq!(count(home), 1);
    ok(home, &["neuron", "revoke", &id]);
    let mut retry = link.to_vec();
    retry.extend(["--attachment", &id]);
    ok(home, &retry); // Revocation clears selection; explicit historical attachment remains readable.
    assert_eq!(peer.count(), 1);
    assert_eq!(count(home), 1);
    ok(
        home,
        &[
            "relay",
            "--attachment",
            &id,
            "--network",
            &network,
            "--rpc",
            &peer.url,
        ],
    );
    assert_eq!(peer.count(), 1);
}
fn prepared(home: &Path, network: [u8; 32], request: [u8; 32], attempt: bool) {
    let session = GraphSession::open(home.join("log")).unwrap();
    let registry = Registry::from_database(session.database()).unwrap();
    let host = Host::new(home.into(), home.into(), "true-cyber".into()).unwrap();
    host.execute(&registry, &["create", "a", &hex(&network)])
        .unwrap();
    let a = registry.robot().unwrap().selected().unwrap().clone();
    let key = custody::load(Path::new(a.key_ref.as_deref().unwrap())).unwrap();
    let mut signal = session.prepare_signal(
        key.handle.subject(),
        network,
        [(
            *hemera::hash(b"alpha").as_bytes(),
            *hemera::hash(b"beta").as_bytes(),
            1,
        )],
    );
    signal.links[0].valence = 0;
    let action = registry
        .admit(
            request,
            a.id,
            "native/signal",
            foculus::signal_codec::encode_signal(&signal).unwrap(),
        )
        .unwrap();
    if attempt {
        registry.dispatch(&action, &key.handle, |_, _| ()).unwrap();
    }
}
#[test]
fn missing_local_receipts_recover_without_preparing_a_new_sequence() {
    for attempt in [false, true] {
        let peer = Peer::new(200 + u64::from(attempt));
        let home = tempfile::tempdir().unwrap();
        let request = [13; 32];
        prepared(home.path(), peer.network, request, attempt);
        ok(
            home.path(),
            &[
                "link",
                "alpha",
                "beta",
                "--network",
                &hex(&peer.network),
                "--rpc",
                &peer.url,
                "--request",
                &hex(&request),
            ],
        );
        assert_eq!(peer.count(), 1);
        assert_eq!(count(home.path()), 1);
        ok(
            home.path(),
            &[
                "link",
                "alpha",
                "beta",
                "--network",
                &hex(&peer.network),
                "--rpc",
                &peer.url,
                "--request",
                &hex(&request),
            ],
        );
        assert_eq!(peer.count(), 1);
    }
}
#[test]
fn old_log_migrates_exactly_and_bad_input_never_creates_authority() {
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    let mut source = GraphSession::ephemeral();
    source.link([1; 32], [2; 32], [3; 32]).unwrap();
    source.link([4; 32], [3; 32], [5; 32]).unwrap();
    let bytes: Vec<_> = source
        .signals()
        .into_iter()
        .flat_map(foculus::encode_signal_frame)
        .collect();
    std::fs::write(home.join("log"), &bytes).unwrap();
    let value: Value = serde_json::from_str(&ok(home, &["status"])).unwrap();
    assert_eq!(value["signals"], 2);
    assert_eq!(value["attachments"], 0);
    assert_eq!(value["legacy"]["retired"], true);
    assert_eq!(value["consensus_verified"], false);
    assert_eq!(std::fs::read(home.join("log/source")).unwrap(), bytes);
    assert!(!home.join("keys").exists());
    assert!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(home.join("log"))
            .is_err()
    );
    let export = home.join("export.frames");
    ok(home, &["legacy", "export", export.to_str().unwrap()]);
    assert_eq!(std::fs::read(&export).unwrap(), bytes);
    bad(home, &["legacy", "export", export.to_str().unwrap()]);
    assert_eq!(std::fs::read(&export).unwrap(), bytes);
    assert_eq!(count(home), 2);
    let broken = tempfile::tempdir().unwrap();
    std::fs::write(broken.path().join("log"), &bytes[..bytes.len() - 1]).unwrap();
    bad(broken.path(), &["status"]);
    assert!(!broken.path().join("log.bbg").exists());
    let empty = home.join("invalid-options");
    bad(&empty, &["link", "a", "b", "--neuron", "01"]);
    bad(&empty, &["sync"]);
    bad(&empty, &["sync", "--network", "01"]);
    bad(&empty, &["unknown"]);
    assert!(!empty.exists());
}
