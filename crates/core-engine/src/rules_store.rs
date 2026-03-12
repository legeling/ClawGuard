use crate::Ruleset;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const RULES_SCHEMA_VERSION: u32 = 1;
const STORE_STATE_FILE: &str = "state.json";
const PACKS_DIR: &str = "packs";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesPackPayload {
    pub schema_version: u32,
    pub pack_version: String,
    pub created_at_epoch_s: u64,
    pub min_engine_version: String,
    pub rules: Ruleset,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedRulesPack {
    pub key_id: String,
    pub payload: RulesPackPayload,
    pub signature_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedRulesPack {
    pub version: String,
    pub key_id: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulesActivationOutcome {
    pub active_version: String,
    pub previous_version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulesStoreStatus {
    pub active_version: Option<String>,
    pub installed_versions: Vec<String>,
    pub activation_history: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct RulesStoreState {
    active_version: Option<String>,
    activation_history: Vec<String>,
}

pub fn default_rules_store_dir() -> PathBuf {
    std::env::var_os("CLAWGUARD_RULES_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join(".clawguard").join("rules"))
        })
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|home| PathBuf::from(home).join(".clawguard").join("rules"))
        })
        .unwrap_or_else(|| PathBuf::from(".clawguard").join("rules"))
}

pub fn generate_signing_keypair_hex() -> (String, String) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    (
        encode_hex(signing_key.to_bytes().as_slice()),
        encode_hex(verifying_key.to_bytes().as_slice()),
    )
}

pub fn create_signed_rules_pack(
    rules: Ruleset,
    pack_version: &str,
    key_id: &str,
    private_key_hex: &str,
) -> Result<SignedRulesPack, String> {
    validate_pack_version(pack_version)?;

    let signing_key = signing_key_from_hex(private_key_hex)?;
    let payload = RulesPackPayload {
        schema_version: RULES_SCHEMA_VERSION,
        pack_version: pack_version.to_string(),
        created_at_epoch_s: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0),
        min_engine_version: current_engine_version().to_string(),
        rules,
    };
    let signature = signing_key.sign(&canonical_pack_payload(&payload)?);

    Ok(SignedRulesPack {
        key_id: key_id.to_string(),
        payload,
        signature_hex: encode_hex(signature.to_bytes().as_slice()),
    })
}

pub fn render_rules_pack_json(pack: &SignedRulesPack) -> Result<String, String> {
    serde_json::to_string_pretty(pack)
        .map_err(|error| format!("failed to serialize signed rules pack: {error}"))
}

pub fn write_rules_pack(path: &Path, pack: &SignedRulesPack) -> Result<(), String> {
    let content = render_rules_pack_json(pack)?;
    fs::write(path, content).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

pub fn load_rules_pack(path: &Path) -> Result<SignedRulesPack, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    parse_rules_pack(&content)
}

pub fn parse_rules_pack(input: &str) -> Result<SignedRulesPack, String> {
    serde_json::from_str(input).map_err(|error| format!("failed to parse signed rules pack: {error}"))
}

pub fn verify_rules_pack(pack: &SignedRulesPack, public_key_hex: &str) -> Result<(), String> {
    validate_pack_version(&pack.payload.pack_version)?;
    if pack.payload.schema_version != RULES_SCHEMA_VERSION {
        return Err(format!(
            "unsupported rules schema version: {}",
            pack.payload.schema_version
        ));
    }

    if compare_semver(current_engine_version(), &pack.payload.min_engine_version)? == Ordering::Less {
        return Err(format!(
            "rules pack requires engine version {} or newer",
            pack.payload.min_engine_version
        ));
    }

    let verifying_key = verifying_key_from_hex(public_key_hex)?;
    let signature = signature_from_hex(&pack.signature_hex)?;
    verifying_key
        .verify(&canonical_pack_payload(&pack.payload)?, &signature)
        .map_err(|error| format!("rules pack signature verification failed: {error}"))
}

pub fn import_rules_pack(
    pack_path: &Path,
    public_key_hex: &str,
    store_dir: &Path,
) -> Result<ImportedRulesPack, String> {
    let pack = load_rules_pack(pack_path)?;
    verify_rules_pack(&pack, public_key_hex)?;

    let destination = pack_path_for(store_dir, &pack.payload.pack_version)?;
    fs::create_dir_all(packs_dir(store_dir))
        .map_err(|error| format!("failed to create {}: {error}", packs_dir(store_dir).display()))?;
    write_rules_pack(&destination, &pack)?;

    Ok(ImportedRulesPack {
        version: pack.payload.pack_version,
        key_id: pack.key_id,
        path: destination,
    })
}

pub fn activate_rules_pack(
    store_dir: &Path,
    version: &str,
) -> Result<RulesActivationOutcome, String> {
    let pack_path = pack_path_for(store_dir, version)?;
    if !pack_path.exists() {
        return Err(format!(
            "rules pack version {version} is not installed in {}",
            store_dir.display()
        ));
    }

    let mut state = load_store_state(store_dir)?;
    let previous_version = state.active_version.clone();

    if previous_version.as_deref() != Some(version) {
        if let Some(previous) = previous_version.as_ref() {
            state.activation_history.push(previous.clone());
        }
        state.active_version = Some(version.to_string());
    }

    write_store_state(store_dir, &state)?;

    Ok(RulesActivationOutcome {
        active_version: version.to_string(),
        previous_version,
    })
}

pub fn rollback_rules_pack(store_dir: &Path) -> Result<RulesActivationOutcome, String> {
    let mut state = load_store_state(store_dir)?;
    let current = state
        .active_version
        .clone()
        .ok_or_else(|| format!("no active rules pack in {}", store_dir.display()))?;

    while let Some(candidate) = state.activation_history.pop() {
        let candidate_path = pack_path_for(store_dir, &candidate)?;
        if candidate == current || !candidate_path.exists() {
            continue;
        }

        state.active_version = Some(candidate.clone());
        write_store_state(store_dir, &state)?;

        return Ok(RulesActivationOutcome {
            active_version: candidate,
            previous_version: Some(current),
        });
    }

    Err(format!(
        "no rollback target is available in {}",
        store_dir.display()
    ))
}

pub fn rules_store_status(store_dir: &Path) -> Result<RulesStoreStatus, String> {
    let state = load_store_state(store_dir)?;
    let mut installed_versions = Vec::new();
    let packs_dir = packs_dir(store_dir);

    if packs_dir.exists() {
        let entries = fs::read_dir(&packs_dir)
            .map_err(|error| format!("failed to read {}: {error}", packs_dir.display()))?;

        for entry in entries {
            let entry = entry.map_err(|error| format!("failed to read rules pack entry: {error}"))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            if let Some(name) = path.file_stem().and_then(|value| value.to_str()) {
                installed_versions.push(name.to_string());
            }
        }
    }

    installed_versions.sort();

    Ok(RulesStoreStatus {
        active_version: state.active_version,
        installed_versions,
        activation_history: state.activation_history,
    })
}

pub fn load_active_ruleset(store_dir: &Path) -> Result<Option<Ruleset>, String> {
    let state = load_store_state(store_dir)?;
    let Some(active_version) = state.active_version else {
        return Ok(None);
    };

    let pack = load_rules_pack(&pack_path_for(store_dir, &active_version)?)?;
    Ok(Some(pack.payload.rules))
}

fn current_engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn canonical_pack_payload(payload: &RulesPackPayload) -> Result<Vec<u8>, String> {
    serde_json::to_vec(payload)
        .map_err(|error| format!("failed to canonicalize rules payload: {error}"))
}

fn load_store_state(store_dir: &Path) -> Result<RulesStoreState, String> {
    let state_path = store_dir.join(STORE_STATE_FILE);
    if !state_path.exists() {
        return Ok(RulesStoreState::default());
    }

    let content = fs::read_to_string(&state_path)
        .map_err(|error| format!("failed to read {}: {error}", state_path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("failed to parse {}: {error}", state_path.display()))
}

fn write_store_state(store_dir: &Path, state: &RulesStoreState) -> Result<(), String> {
    fs::create_dir_all(store_dir)
        .map_err(|error| format!("failed to create {}: {error}", store_dir.display()))?;
    let state_path = store_dir.join(STORE_STATE_FILE);
    let content = serde_json::to_string_pretty(state)
        .map_err(|error| format!("failed to serialize rules store state: {error}"))?;
    fs::write(&state_path, content)
        .map_err(|error| format!("failed to write {}: {error}", state_path.display()))
}

fn packs_dir(store_dir: &Path) -> PathBuf {
    store_dir.join(PACKS_DIR)
}

fn pack_path_for(store_dir: &Path, version: &str) -> Result<PathBuf, String> {
    validate_pack_version(version)?;
    Ok(packs_dir(store_dir).join(format!("{version}.json")))
}

fn validate_pack_version(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("rules pack version cannot be empty".to_string());
    }

    if !version
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        return Err(format!(
            "rules pack version contains unsupported characters: {version}"
        ));
    }

    Ok(())
}

fn signing_key_from_hex(value: &str) -> Result<SigningKey, String> {
    let bytes = decode_hex(value.trim())?;
    let secret: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "signing key must contain exactly 32 bytes".to_string())?;
    Ok(SigningKey::from_bytes(&secret))
}

fn verifying_key_from_hex(value: &str) -> Result<VerifyingKey, String> {
    let bytes = decode_hex(value.trim())?;
    let public: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "verifying key must contain exactly 32 bytes".to_string())?;
    VerifyingKey::from_bytes(&public)
        .map_err(|error| format!("invalid verifying key: {error}"))
}

fn signature_from_hex(value: &str) -> Result<Signature, String> {
    let bytes = decode_hex(value.trim())?;
    let signature: [u8; 64] = bytes
        .try_into()
        .map_err(|_| "signature must contain exactly 64 bytes".to_string())?;
    Ok(Signature::from_bytes(&signature))
}

fn compare_semver(current: &str, required: &str) -> Result<Ordering, String> {
    let current = normalize_semver(current)?;
    let required = normalize_semver(required)?;
    Ok(current.cmp(&required))
}

fn normalize_semver(input: &str) -> Result<[u64; 3], String> {
    let normalized = input.trim().split('-').next().unwrap_or(input.trim());
    let mut version = [0_u64; 3];

    for (index, part) in normalized.split('.').take(3).enumerate() {
        version[index] = part
            .parse::<u64>()
            .map_err(|_| format!("invalid semantic version: {input}"))?;
    }

    Ok(version)
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }
    output
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must be in range 0..=15"),
    }
}

fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    if !trimmed.len().is_multiple_of(2) {
        return Err("hex input must have an even number of characters".to_string());
    }

    let mut bytes = Vec::with_capacity(trimmed.len() / 2);
    let chars = trimmed.as_bytes().chunks_exact(2);

    for pair in chars {
        let high = decode_hex_nibble(pair[0])?;
        let low = decode_hex_nibble(pair[1])?;
        bytes.push((high << 4) | low);
    }

    Ok(bytes)
}

fn decode_hex_nibble(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("hex input contains a non-hex character".to_string()),
    }
}
