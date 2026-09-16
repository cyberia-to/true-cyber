use crate::{Result, error, hex};
use cybergraph::legacy_file::LegacyFile;
use serde_json::{Value, json};
use std::{io::Write, path::Path};
fn load(home: &Path) -> Result<Option<LegacyFile>> {
    LegacyFile::load(
        &home.join("log"),
        "cyb/signal-log/1",
        foculus::frames::MAX_LEGACY_LOG_BYTES,
    )
    .map_err(error)
}
pub fn inspect(home: &Path) -> Result<Value> {
    let Some(source) = load(home)? else {
        return Ok(Value::Null);
    };
    let events = foculus::frames::decode_events_strict(source.bytes()).map_err(error)?;
    Ok(
        json!({"schema":"true-cyber/legacy-origin/1","retired":source.is_retired(),
        "bytes":source.bytes().len(),"source_hash":hex(hemera::hash(source.bytes()).as_bytes()),"events":events.len(),
        "identity":"original unsigned log authors; no custody inferred","network":"original frame bytes; no authenticated genesis pin"}),
    )
}
pub fn export(home: &Path, path: &Path) -> Result<()> {
    let source = load(home)?.ok_or("no retained legacy source")?;
    // Exact forensic export, including bytes which fail decoding.
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(error)?;
    file.write_all(source.bytes())
        .and_then(|_| file.sync_all())
        .map_err(error)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(error)?;
    println!(
        "{} bytes exported; source {}",
        source.bytes().len(),
        hex(hemera::hash(source.bytes()).as_bytes())
    );
    Ok(())
}
