
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::Aes256Gcm;
use chacha20poly1305::ChaCha20Poly1305;

pub mod sync;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;
pub const SALT_LEN: usize = 32;
pub const MASTER_LEN: usize = 64;
pub const ARGON_M_COST: u32 = 65536;
pub const ARGON_T_COST: u32 = 3;
pub const ARGON_P_COST: u32 = 4;
pub const ACCOUNT_KEY_LEN: usize = 32;
pub const VAULT_VERSION: u32 = 5;
pub const CONTAINER_VERSION: u8 = 3;
pub const VAULT_PART_LEN: usize = 32;
pub const MAGIC: &[u8; 4] = b"SVLT";
const KEYCHAIN_SERVICE: &str = "SecureVault";

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex(&h.finalize())
}

fn sha256_32(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    let d = h.finalize();
    let mut a = [0u8; 32];
    a.copy_from_slice(&d);
    a
}

fn merkle_root_hex(chunks: &BTreeMap<String, String>) -> Option<String> {
    if chunks.is_empty() {
        return None;
    }
    let mut level: Vec<[u8; 32]> = chunks
        .iter()
        .map(|(id, blob)| sha256_32(&[id.as_bytes(), b"\0", blob.as_bytes()]))
        .collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        let mut i = 0;
        while i < level.len() {
            let left = &level[i];
            let right = if i + 1 < level.len() { &level[i + 1] } else { &level[i] };
            next.push(sha256_32(&[left, right]));
            i += 2;
        }
        level = next;
    }
    Some(hex(&level[0]))
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "this-device".into())
}

fn random_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    getrandom::getrandom(&mut buf).expect("сбой CSPRNG ОС");
    buf
}

fn mix_entropy(buf: &mut [u8], extra: &[u8]) {
    if extra.is_empty() {
        return;
    }
    for (i, b) in buf.iter_mut().enumerate() {
        *b ^= extra[i % extra.len()];
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn parse_hex(s: &str) -> Result<Vec<u8>> {
    let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.is_empty() || !clean.len().is_multiple_of(2) {
        return Err(anyhow!("неверный формат Account Key"));
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).map_err(|e| anyhow!("{e}")))
        .collect()
}

fn group(s: &str, n: usize) -> String {
    s.as_bytes()
        .chunks(n)
        .map(|c| std::str::from_utf8(c).unwrap().to_string())
        .collect::<Vec<_>>()
        .join("-")
}

fn format_account_key(ak: &[u8]) -> String {
    group(&hex(ak), 8)
}

fn read_b64_file(p: &Path) -> Option<Vec<u8>> {
    fs::read_to_string(p).ok().and_then(|s| B64.decode(s.trim()).ok())
}

fn integrity_file(p: &Path) -> PathBuf {
    p.with_extension("integrity")
}

fn file_hash(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    let mut h = Sha256::new();
    h.update(&data);
    Some(hex(h.finalize().as_slice()))
}

#[derive(Clone)]
pub enum DeviceStore {

    File(PathBuf),

    Keychain { id: String, legacy_file: PathBuf },
}

impl DeviceStore {
    pub fn file<P: AsRef<Path>>(p: P) -> Self {
        DeviceStore::File(p.as_ref().to_path_buf())
    }
    pub fn keychain(id: impl Into<String>, legacy_file: impl AsRef<Path>) -> Self {
        DeviceStore::Keychain { id: id.into(), legacy_file: legacy_file.as_ref().to_path_buf() }
    }

    fn entry(id: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(KEYCHAIN_SERVICE, id).map_err(|e| anyhow!("keychain: {e}"))
    }

    pub fn load(&self) -> Option<Vec<u8>> {
        match self {
            DeviceStore::File(p) => read_b64_file(p),
            DeviceStore::Keychain { id, legacy_file } => {
                if let Ok(e) = Self::entry(id) {
                    if let Ok(s) = e.get_password() {
                        if let Ok(b) = B64.decode(s.trim()) {
                            return Some(b);
                        }
                    }
                }

                if let Some(b) = read_b64_file(legacy_file) {
                    if self.store(&b).is_ok() {
                        let _ = fs::remove_file(legacy_file);
                    }
                    return Some(b);
                }
                None
            }
        }
    }

    pub fn store(&self, ak: &[u8]) -> Result<()> {
        let b = B64.encode(ak);
        match self {
            DeviceStore::File(p) => fs::write(p, b).context("не могу сохранить ключ на устройстве"),
            DeviceStore::Keychain { id, .. } => {
                Self::entry(id)?.set_password(&b).map_err(|e| anyhow!("keychain: {e}"))
            }
        }
    }

    pub fn clear(&self) -> Result<()> {
        self.clear_integrity();
        match self {
            DeviceStore::File(p) => {
                if p.exists() {
                    fs::remove_file(p).context("не могу удалить ключ устройства")?;
                }
                Ok(())
            }
            DeviceStore::Keychain { id, legacy_file } => {
                if legacy_file.exists() {
                    let _ = fs::remove_file(legacy_file);
                }
                match Self::entry(id)?.delete_credential() {
                    Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                    Err(e) => Err(anyhow!("keychain: {e}")),
                }
            }
        }
    }

    pub fn exists(&self) -> bool {
        self.load().is_some()
    }

    pub fn load_integrity(&self) -> Option<String> {
        match self {
            DeviceStore::File(p) => fs::read_to_string(integrity_file(p)).ok().map(|s| s.trim().to_string()),
            DeviceStore::Keychain { id, .. } => Self::entry(&format!("{id}::integrity")).ok()?.get_password().ok(),
        }
    }

    pub fn store_integrity(&self, h: &str) {
        match self {
            DeviceStore::File(p) => {
                let _ = fs::write(integrity_file(p), h);
            }
            DeviceStore::Keychain { id, .. } => {
                if let Ok(e) = Self::entry(&format!("{id}::integrity")) {
                    let _ = e.set_password(h);
                }
            }
        }
    }

    fn clear_integrity(&self) {
        match self {
            DeviceStore::File(p) => {
                let _ = fs::remove_file(integrity_file(p));
            }
            DeviceStore::Keychain { id, .. } => {
                if let Ok(e) = Self::entry(&format!("{id}::integrity")) {
                    let _ = e.delete_credential();
                }
            }
        }
    }

    pub fn describe(&self) -> String {
        match self {
            DeviceStore::File(p) => format!("файл {}", p.display()),
            DeviceStore::Keychain { .. } => "защищённое хранилище ОС (Credential Manager)".into(),
        }
    }
}

pub struct DualKey {
    pub aes: Zeroizing<[u8; KEY_LEN]>,
    pub chacha: Zeroizing<[u8; KEY_LEN]>,
}

impl DualKey {
    fn random() -> Self {
        let mut aes = [0u8; KEY_LEN];
        let mut chacha = [0u8; KEY_LEN];
        getrandom::getrandom(&mut aes).expect("CSPRNG");
        getrandom::getrandom(&mut chacha).expect("CSPRNG");
        DualKey { aes: Zeroizing::new(aes), chacha: Zeroizing::new(chacha) }
    }
    fn to_bytes(&self) -> Zeroizing<[u8; MASTER_LEN]> {
        let mut b = [0u8; MASTER_LEN];
        b[..KEY_LEN].copy_from_slice(&*self.aes);
        b[KEY_LEN..].copy_from_slice(&*self.chacha);
        Zeroizing::new(b)
    }
    fn from_bytes(b: &[u8]) -> Result<Self> {
        if b.len() != MASTER_LEN {
            return Err(anyhow!("неверная длина ключевого материала"));
        }
        let mut aes = [0u8; KEY_LEN];
        let mut chacha = [0u8; KEY_LEN];
        aes.copy_from_slice(&b[..KEY_LEN]);
        chacha.copy_from_slice(&b[KEY_LEN..]);
        Ok(DualKey { aes: Zeroizing::new(aes), chacha: Zeroizing::new(chacha) })
    }
}

fn derive_kek(secret: &[u8], salt: &[u8]) -> Result<DualKey> {
    use argon2::{Algorithm, Argon2, Params, Version};
    let params = Params::new(ARGON_M_COST, ARGON_T_COST, ARGON_P_COST, Some(MASTER_LEN))
        .map_err(|e| anyhow!("параметры argon2: {e}"))?;
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = Zeroizing::new([0u8; MASTER_LEN]);
    a2.hash_password_into(secret, salt, out.as_mut_slice())
        .map_err(|e| anyhow!("argon2: {e}"))?;
    DualKey::from_bytes(&*out)
}

fn unlock_kek(password: &[u8], account_key: Option<&[u8]>, salt: &[u8]) -> Result<DualKey> {
    match account_key {
        None => derive_kek(password, salt),
        Some(ak) => {
            let mut secret = Zeroizing::new(Vec::with_capacity(password.len() + ak.len()));
            secret.extend_from_slice(password);
            secret.extend_from_slice(ak);
            derive_kek(secret.as_slice(), salt)
        }
    }
}

fn master_kek(password: &[u8], vault_part: &[u8], account_key: Option<&[u8]>, salt: &[u8]) -> Result<DualKey> {
    let mut secret = Zeroizing::new(Vec::with_capacity(password.len() + vault_part.len() + ACCOUNT_KEY_LEN));
    secret.extend_from_slice(password);
    secret.extend_from_slice(vault_part);
    if let Some(ak) = account_key {
        secret.extend_from_slice(ak);
    }
    derive_kek(secret.as_slice(), salt)
}

pub fn encrypt_blob(plaintext: &[u8], key: &DualKey) -> Result<Vec<u8>> {
    let compressed = zstd::encode_all(plaintext, 9).context("zstd сжатие")?;
    let nonce_aes = random_bytes(NONCE_LEN);
    let aes = Aes256Gcm::new_from_slice(&*key.aes).map_err(|e| anyhow!("ключ AES: {e}"))?;
    let layer1 = aes
        .encrypt(GenericArray::from_slice(&nonce_aes), compressed.as_ref())
        .map_err(|_| anyhow!("ошибка шифрования AES"))?;
    let nonce_cc = random_bytes(NONCE_LEN);
    let cc = ChaCha20Poly1305::new_from_slice(&*key.chacha).map_err(|e| anyhow!("ключ ChaCha: {e}"))?;
    let layer2 = cc
        .encrypt(GenericArray::from_slice(&nonce_cc), layer1.as_ref())
        .map_err(|_| anyhow!("ошибка шифрования ChaCha20"))?;
    let mut blob = Vec::with_capacity(NONCE_LEN * 2 + layer2.len());
    blob.extend_from_slice(&nonce_aes);
    blob.extend_from_slice(&nonce_cc);
    blob.extend_from_slice(&layer2);
    Ok(blob)
}

pub fn decrypt_blob(blob: &[u8], key: &DualKey) -> Result<Zeroizing<Vec<u8>>> {
    if blob.len() < NONCE_LEN * 2 + 32 {
        return Err(anyhow!("blob слишком короткий / повреждён"));
    }
    let nonce_aes = &blob[..NONCE_LEN];
    let nonce_cc = &blob[NONCE_LEN..NONCE_LEN * 2];
    let layer2 = &blob[NONCE_LEN * 2..];
    let cc = ChaCha20Poly1305::new_from_slice(&*key.chacha).map_err(|e| anyhow!("ключ ChaCha: {e}"))?;
    let layer1 = cc
        .decrypt(GenericArray::from_slice(nonce_cc), layer2)
        .map_err(|_| anyhow!("ChaCha20: неверный ключ или данные подделаны"))?;
    let aes = Aes256Gcm::new_from_slice(&*key.aes).map_err(|e| anyhow!("ключ AES: {e}"))?;
    let compressed = aes
        .decrypt(GenericArray::from_slice(nonce_aes), layer1.as_ref())
        .map_err(|_| anyhow!("AES: неверный ключ или данные подделаны"))?;
    let plaintext = zstd::decode_all(compressed.as_slice()).context("zstd распаковка")?;
    Ok(Zeroizing::new(plaintext))
}

fn wrap_key(vk: &DualKey, kek: &DualKey) -> Result<String> {
    Ok(B64.encode(encrypt_blob(vk.to_bytes().as_ref(), kek)?))
}
fn unwrap_key(wrapped: &str, kek: &DualKey) -> Result<DualKey> {
    let blob = B64.decode(wrapped).context("повреждённый конверт ключа")?;
    DualKey::from_bytes(&decrypt_blob(&blob, kek)?)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Record {
    pub id: String,
    #[serde(rename = "type")]
    pub rec_type: String,
    pub title: String,
    pub search_hints: Vec<String>,
    pub created: String,
    pub modified: String,
    pub size_bytes: usize,
    pub chunk_key_encrypted: String,
}

#[derive(Serialize, Deserialize)]
pub struct Index {
    pub version: u32,
    pub records: Vec<Record>,
}

#[derive(Serialize, Deserialize, Clone)]
struct KdfParams {
    alg: String,
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
    salt: String,
}

#[derive(Serialize, Deserialize)]
struct VaultFile {
    version: u32,
    kdf: KdfParams,
    wrap: String,

    #[serde(default)]
    ak_enabled: bool,
    index_blob: String,
    chunks: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionSettings {
    pub trust_duration_days: u32,
    pub auto_lock_minutes: u32,
}
impl Default for SessionSettings {
    fn default() -> Self {
        SessionSettings { trust_duration_days: 7, auto_lock_minutes: 15 }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Permissions {
    pub delete_files: bool,
    pub manage_devices: bool,
    pub reset_trust: bool,
    pub totp_manage: bool,
    pub view_only: bool,
}
impl Permissions {

    pub fn full() -> Self {
        Permissions { delete_files: true, manage_devices: true, reset_trust: true, totp_manage: true, view_only: false }
    }

    pub fn standard() -> Self {
        Permissions { delete_files: false, manage_devices: false, reset_trust: false, totp_manage: false, view_only: false }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceEntry {
    pub id: String,
    pub name: String,
    pub permissions: Permissions,
    pub trust_expires: Option<String>,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StorageEntry {
    pub storage_id: String,
    pub binding_token: String,
}

#[derive(Serialize, Deserialize)]
struct PayloadV3 {
    vault_part: String,
    ak_enabled: bool,
    account_key_hash: Option<String>,
    wrap: String,
    session: SessionSettings,
    devices: Vec<DeviceEntry>,
    storages: Vec<StorageEntry>,
    merkle_root: Option<String>,
    index_blob: String,
    chunks: BTreeMap<String, String>,
}

pub struct Vault {
    vault_path: PathBuf,
    device_store: DeviceStore,
    salt: Vec<u8>,
    hk: DualKey,
    wrapped_hk: Vec<u8>,
    vault_part: Vec<u8>,
    vk: DualKey,

    account_key: Option<Zeroizing<Vec<u8>>>,
    ak_enabled: bool,
    account_key_hash: Option<String>,
    wrap: String,
    session: SessionSettings,
    devices: Vec<DeviceEntry>,
    storages: Vec<StorageEntry>,
    index: Index,
    chunks: BTreeMap<String, String>,
}

impl Vault {
    pub fn is_trusted(device_store: &DeviceStore) -> bool {
        device_store.exists()
    }

    pub fn init<P: AsRef<Path>>(
        vault_path: P,
        device_store: DeviceStore,
        password: &[u8],
    ) -> Result<Vault> {
        let vault_path = vault_path.as_ref().to_path_buf();
        if vault_path.exists() {
            return Err(anyhow!("хранилище уже существует: {}", vault_path.display()));
        }
        let salt = random_bytes(SALT_LEN);
        let vault_part = random_bytes(VAULT_PART_LEN);
        let hk = DualKey::random();
        let vk = DualKey::random();
        let header_kek = derive_kek(password, &salt)?;
        let wrapped_hk = encrypt_blob(hk.to_bytes().as_ref(), &header_kek)?;
        let master = master_kek(password, &vault_part, None, &salt)?;
        let wrap = wrap_key(&vk, &master)?;
        let primary = DeviceEntry {
            id: hex(&random_bytes(8)),
            name: device_name(),
            permissions: Permissions::full(),
            trust_expires: None,
            status: "primary".into(),
        };
        let v = Vault {
            vault_path,
            device_store,
            salt,
            hk,
            wrapped_hk,
            vault_part,
            vk,
            account_key: None,
            ak_enabled: false,
            account_key_hash: None,
            wrap,
            session: SessionSettings::default(),
            devices: vec![primary],
            storages: vec![],
            index: Index { version: 2, records: vec![] },
            chunks: BTreeMap::new(),
        };
        v.save()?;
        Ok(v)
    }

    pub fn open<P: AsRef<Path>>(vault_path: P, device_store: DeviceStore, password: &[u8]) -> Result<Vault> {
        let vault_path = vault_path.as_ref().to_path_buf();
        let raw = fs::read(&vault_path)
            .with_context(|| format!("не могу прочитать хранилище: {}", vault_path.display()))?;
        if raw.starts_with(MAGIC) {
            Self::open_v3(vault_path, device_store, password, None, true)
        } else {

            Self::migrate_open(vault_path, device_store, password)
        }
    }

    fn open_v3(
        vault_path: PathBuf,
        device_store: DeviceStore,
        password: &[u8],
        account_key_input: Option<&str>,
        trust: bool,
    ) -> Result<Vault> {
        let (_flags, salt, wrapped_hk, payload_cipher) = load_container_or_recover(&vault_path)?;
        let header_kek = derive_kek(password, &salt)?;
        let hk = DualKey::from_bytes(
            &decrypt_blob(&wrapped_hk, &header_kek).map_err(|_| anyhow!("Неверный пароль"))?,
        )?;
        let payload_json = decrypt_blob(&payload_cipher, &hk).map_err(|_| anyhow!("Неверный пароль"))?;
        let payload: PayloadV3 = serde_json::from_slice(&payload_json).context("повреждённый заголовок vault")?;
        let vault_part = B64.decode(&payload.vault_part).context("повреждённый vault_part")?;

        let account_key: Option<Zeroizing<Vec<u8>>> = if payload.ak_enabled {
            match account_key_input {
                Some(input) => {
                    let ak = parse_hex(input)?;
                    if ak.len() != ACCOUNT_KEY_LEN {
                        return Err(anyhow!("неверная длина Account Key"));
                    }
                    Some(Zeroizing::new(ak))
                }
                None => {
                    let ak = device_store.load().ok_or_else(|| {
                        anyhow!("это устройство не доверенное — войди командой `unlock` (нужны пароль и Account Key)")
                    })?;
                    Some(Zeroizing::new(ak))
                }
            }
        } else {
            None
        };

        let master = master_kek(password, &vault_part, account_key.as_ref().map(|z| z.as_slice()), &salt)?;
        let vk = unwrap_key(&payload.wrap, &master).map_err(|_| {
            if account_key_input.is_some() {
                anyhow!("Неверный пароль или Account Key")
            } else {
                anyhow!("Неверный пароль")
            }
        })?;
        let index = decrypt_index(&payload.index_blob, &vk)?;

        if trust && payload.ak_enabled && account_key_input.is_some() {
            if let Some(ak) = account_key.as_ref() {
                let _ = device_store.store(ak.as_slice());
            }
        }

        Ok(Vault {
            vault_path,
            device_store,
            salt,
            hk,
            wrapped_hk,
            vault_part,
            vk,
            account_key,
            ak_enabled: payload.ak_enabled,
            account_key_hash: payload.account_key_hash,
            wrap: payload.wrap,
            session: payload.session,
            devices: payload.devices,
            storages: payload.storages,
            index,
            chunks: payload.chunks,
        })
    }

    fn migrate_open(vault_path: PathBuf, device_store: DeviceStore, password: &[u8]) -> Result<Vault> {
        let vf = load_vaultfile_or_recover(&vault_path)?;
        let salt = B64.decode(&vf.kdf.salt).context("повреждённая соль")?;
        let account_key: Option<Zeroizing<Vec<u8>>> = if vf.ak_enabled {
            let ak = device_store.load().ok_or_else(|| {
                anyhow!("это устройство не доверенное — войди командой `unlock` (нужны пароль и Account Key)")
            })?;
            Some(Zeroizing::new(ak))
        } else {
            None
        };
        let old_kek = unlock_kek(password, account_key.as_ref().map(|z| z.as_slice()), &salt)?;
        let vk = unwrap_key(&vf.wrap, &old_kek).map_err(|_| anyhow!("Неверный пароль"))?;
        let index = decrypt_index(&vf.index_blob, &vk)?;

        let vault_part = random_bytes(VAULT_PART_LEN);
        let hk = DualKey::random();
        let header_kek = derive_kek(password, &salt)?;
        let wrapped_hk = encrypt_blob(hk.to_bytes().as_ref(), &header_kek)?;
        let master = master_kek(password, &vault_part, account_key.as_ref().map(|z| z.as_slice()), &salt)?;
        let wrap = wrap_key(&vk, &master)?;
        let account_key_hash = account_key.as_ref().map(|ak| sha256_hex(ak.as_slice()));
        let primary = DeviceEntry {
            id: hex(&random_bytes(8)),
            name: device_name(),
            permissions: Permissions::full(),
            trust_expires: None,
            status: "primary".into(),
        };
        let v = Vault {
            vault_path,
            device_store,
            salt,
            hk,
            wrapped_hk,
            vault_part,
            vk,
            account_key,
            ak_enabled: vf.ak_enabled,
            account_key_hash,
            wrap,
            session: SessionSettings::default(),
            devices: vec![primary],
            storages: vec![],
            index,
            chunks: vf.chunks,
        };
        v.save()?;
        Ok(v)
    }

    pub fn unlock<P: AsRef<Path>>(
        vault_path: P,
        device_store: DeviceStore,
        password: &[u8],
        account_key_input: &str,
        trust: bool,
    ) -> Result<Vault> {
        let vault_path = vault_path.as_ref().to_path_buf();
        let raw = fs::read(&vault_path)
            .with_context(|| format!("не могу прочитать хранилище: {}", vault_path.display()))?;
        if !raw.starts_with(MAGIC) {

            return Self::migrate_open(vault_path, device_store, password);
        }
        if !vault_uses_account_key(&vault_path.to_string_lossy()) {
            return Err(anyhow!("для этого хранилища Account Key не используется — войдите паролем"));
        }
        Self::open_v3(vault_path, device_store, password, Some(account_key_input), trust)
    }

    pub fn open_or_init<P: AsRef<Path>>(
        vault_path: P,
        device_store: DeviceStore,
        password: &[u8],
    ) -> Result<Vault> {
        if vault_path.as_ref().exists() {
            Self::open(vault_path, device_store, password)
        } else {
            Self::init(vault_path, device_store, password)
        }
    }

    pub fn account_key_enabled(&self) -> bool {
        self.ak_enabled
    }

    pub fn session_settings(&self) -> &SessionSettings {
        &self.session
    }

    pub fn devices(&self) -> &[DeviceEntry] {
        &self.devices
    }

    fn account_key_slice(&self) -> Option<&[u8]> {
        self.account_key.as_ref().map(|z| z.as_slice())
    }

    pub fn set_session_settings(&mut self, trust_duration_days: u32, auto_lock_minutes: u32) -> Result<()> {
        self.session.trust_duration_days = trust_duration_days;
        self.session.auto_lock_minutes = auto_lock_minutes;
        self.save()
    }

    pub fn change_password(&mut self, new_password: &[u8], account_key_input: Option<&str>) -> Result<()> {
        if self.ak_enabled {
            match account_key_input {
                Some(k) if self.verify_account_key(k) => {}
                _ => return Err(anyhow!("смена пароля требует Account Key")),
            }
        }

        let header_kek = derive_kek(new_password, &self.salt)?;
        self.wrapped_hk = encrypt_blob(self.hk.to_bytes().as_ref(), &header_kek)?;
        let master = master_kek(new_password, &self.vault_part, self.account_key_slice(), &self.salt)?;
        self.wrap = wrap_key(&self.vk, &master)?;
        self.save()
    }

    pub fn verify_password(&self, password: &[u8]) -> bool {
        match master_kek(password, &self.vault_part, self.account_key_slice(), &self.salt) {
            Ok(kek) => unwrap_key(&self.wrap, &kek).is_ok(),
            Err(_) => false,
        }
    }

    pub fn verify_account_key(&self, input: &str) -> bool {
        match (parse_hex(input), self.account_key.as_ref()) {
            (Ok(b), Some(ak)) => ct_eq(b.as_slice(), ak.as_slice()),
            _ => false,
        }
    }

    pub fn enable_account_key(&mut self, current_password: &[u8], extra: &[u8], trust: bool) -> Result<String> {
        if self.ak_enabled {
            return Err(anyhow!("Account Key уже включён"));
        }
        if !self.verify_password(current_password) {
            return Err(anyhow!("Неверный пароль"));
        }
        let mut ak = random_bytes(ACCOUNT_KEY_LEN);
        mix_entropy(&mut ak, extra);
        let master = master_kek(current_password, &self.vault_part, Some(ak.as_slice()), &self.salt)?;
        self.wrap = wrap_key(&self.vk, &master)?;
        if trust {
            self.device_store.store(&ak)?;
        }
        self.account_key_hash = Some(sha256_hex(&ak));
        let s = format_account_key(&ak);
        self.account_key = Some(Zeroizing::new(ak));
        self.ak_enabled = true;
        self.save()?;
        Ok(s)
    }

    pub fn rotate_account_key(&mut self, current_password: &[u8]) -> Result<String> {
        if !self.ak_enabled {
            return Err(anyhow!("Account Key не включён — сначала включите его в настройках"));
        }
        if !self.verify_password(current_password) {
            return Err(anyhow!("Неверный пароль"));
        }
        let new_ak = random_bytes(ACCOUNT_KEY_LEN);
        let master = master_kek(current_password, &self.vault_part, Some(new_ak.as_slice()), &self.salt)?;
        self.wrap = wrap_key(&self.vk, &master)?;
        self.device_store.store(&new_ak)?;
        self.account_key_hash = Some(sha256_hex(&new_ak));
        let s = format_account_key(&new_ak);
        self.account_key = Some(Zeroizing::new(new_ak));
        self.save()?;
        Ok(s)
    }

    pub fn disable_account_key(&mut self, current_password: &[u8], account_key_input: &str) -> Result<()> {
        if !self.ak_enabled {
            return Err(anyhow!("Account Key не включён"));
        }
        if !self.verify_password(current_password) {
            return Err(anyhow!("Неверный пароль"));
        }
        if !self.verify_account_key(account_key_input) {
            return Err(anyhow!("Неверный Account Key"));
        }
        let master = master_kek(current_password, &self.vault_part, None, &self.salt)?;
        self.wrap = wrap_key(&self.vk, &master)?;
        let _ = self.device_store.clear();
        self.account_key = None;
        self.account_key_hash = None;
        self.ak_enabled = false;
        self.save()?;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let index_json = serde_json::to_vec(&self.index)?;
        let index_blob = encrypt_blob(&index_json, &self.vk)?;
        let payload = PayloadV3 {
            vault_part: B64.encode(&self.vault_part),
            ak_enabled: self.ak_enabled,
            account_key_hash: self.account_key_hash.clone(),
            wrap: self.wrap.clone(),
            session: self.session.clone(),
            devices: self.devices.clone(),
            storages: self.storages.clone(),
            merkle_root: merkle_root_hex(&self.chunks),
            index_blob: B64.encode(&index_blob),
            chunks: self.chunks.clone(),
        };
        let payload_json = serde_json::to_vec(&payload)?;
        let payload_cipher = encrypt_blob(&payload_json, &self.hk)?;
        let bytes = write_container_bytes(&self.salt, &self.wrapped_hk, &payload_cipher, self.ak_enabled);
        let tmp = self.vault_path.with_extension("svault.tmp");
        fs::write(&tmp, &bytes)?;
        fs::rename(&tmp, &self.vault_path)?;
        write_backup(&self.vault_path, &bytes);
        if let Some(h) = file_hash(&self.vault_path) {
            self.device_store.store_integrity(&h);
        }
        Ok(())
    }

    pub fn add_record(
        &mut self,
        rec_type: &str,
        title: &str,
        data: serde_json::Value,
        hints: Vec<String>,
    ) -> Result<String> {
        let mut obj = serde_json::Map::new();
        obj.insert("type".into(), serde_json::Value::String(rec_type.into()));
        obj.insert("title".into(), serde_json::Value::String(title.into()));
        if let serde_json::Value::Object(m) = data {
            for (k, v) in m {
                obj.insert(k, v);
            }
        }
        let plaintext = serde_json::to_vec(&serde_json::Value::Object(obj))?;
        let chunk_key = DualKey::random();
        let blob = encrypt_blob(&plaintext, &chunk_key)?;
        let id = format!("chunk_{}", hex(&random_bytes(8)));
        self.chunks.insert(id.clone(), B64.encode(&blob));
        let sealed = encrypt_blob(chunk_key.to_bytes().as_ref(), &self.vk)?;
        let now = chrono::Utc::now().to_rfc3339();
        self.index.records.push(Record {
            id: id.clone(),
            rec_type: rec_type.into(),
            title: title.into(),
            search_hints: hints,
            created: now.clone(),
            modified: now,
            size_bytes: plaintext.len(),
            chunk_key_encrypted: B64.encode(&sealed),
        });
        self.save()?;
        Ok(id)
    }

    pub fn get_record(&self, id: &str) -> Result<serde_json::Value> {
        let rec = self
            .index
            .records
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| anyhow!("запись не найдена: {id}"))?;
        let sealed = B64.decode(&rec.chunk_key_encrypted)?;
        let key_bytes = decrypt_blob(&sealed, &self.vk)?;
        let chunk_key = DualKey::from_bytes(&key_bytes)?;
        let blob_b64 = self.chunks.get(id).ok_or_else(|| anyhow!("blob отсутствует: {id}"))?;
        let blob = B64.decode(blob_b64)?;
        let plaintext = decrypt_blob(&blob, &chunk_key)?;
        Ok(serde_json::from_slice(&plaintext)?)
    }

    pub fn delete_record(&mut self, id: &str) -> Result<()> {
        let before = self.index.records.len();
        self.index.records.retain(|r| r.id != id);
        if self.index.records.len() == before {
            return Err(anyhow!("запись не найдена: {id}"));
        }
        self.chunks.remove(id);
        self.save()?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<&Record> {
        let q = query.to_lowercase();
        self.index
            .records
            .iter()
            .filter(|r| {
                r.title.to_lowercase().contains(&q)
                    || r.search_hints.iter().any(|h| h.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn records(&self) -> &[Record] {
        &self.index.records
    }
}

pub fn vault_uses_account_key(vault_path: &str) -> bool {
    let p = std::path::Path::new(vault_path);
    if let Ok(bytes) = fs::read(p) {
        if bytes.starts_with(MAGIC) {

            return parse_container(&bytes).map(|(flags, ..)| flags & 1 != 0).unwrap_or(false);
        }
    }

    load_vaultfile_or_recover(p).map(|vf| vf.ak_enabled).unwrap_or(false)
}

pub fn is_vault_file(path: &Path) -> bool {
    use std::io::Read;
    if let Ok(mut f) = fs::File::open(path) {
        let mut buf = [0u8; 4];
        if f.read_exact(&mut buf).is_ok() {
            return &buf == MAGIC;
        }
    }
    false
}

fn write_container_bytes(salt: &[u8], wrapped_hk: &[u8], payload: &[u8], ak_enabled: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(6 + salt.len() + 8 + wrapped_hk.len() + payload.len());
    out.extend_from_slice(MAGIC);
    out.push(CONTAINER_VERSION);
    out.push(if ak_enabled { 1 } else { 0 });
    out.extend_from_slice(salt);
    out.extend_from_slice(&(wrapped_hk.len() as u32).to_le_bytes());
    out.extend_from_slice(wrapped_hk);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

type ContainerParts = (u8, Vec<u8>, Vec<u8>, Vec<u8>);

fn parse_container(bytes: &[u8]) -> Result<ContainerParts> {
    let head = 4 + 1 + 1 + SALT_LEN;
    if bytes.len() < head + 4 || &bytes[..4] != MAGIC {
        return Err(anyhow!("не контейнер SecureVault (нет сигнатуры SVLT)"));
    }
    let flags = bytes[5];
    let mut off = 6;
    let salt = bytes[off..off + SALT_LEN].to_vec();
    off += SALT_LEN;
    let rd = |b: &[u8], o: usize| -> Result<usize> {
        if o + 4 > b.len() {
            return Err(anyhow!("повреждённый контейнер"));
        }
        Ok(u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize)
    };
    let hk_len = rd(bytes, off)?;
    off += 4;
    if off + hk_len > bytes.len() {
        return Err(anyhow!("повреждённый контейнер"));
    }
    let wrapped_hk = bytes[off..off + hk_len].to_vec();
    off += hk_len;
    let pl_len = rd(bytes, off)?;
    off += 4;
    if off + pl_len > bytes.len() {
        return Err(anyhow!("повреждённый контейнер"));
    }
    let payload = bytes[off..off + pl_len].to_vec();
    Ok((flags, salt, wrapped_hk, payload))
}

fn load_container_or_recover(path: &Path) -> Result<ContainerParts> {
    if let Ok(bytes) = fs::read(path) {
        if let Ok(parsed) = parse_container(&bytes) {
            return Ok(parsed);
        }
    }

    let token = read_backup_token(path).ok_or_else(|| {
        anyhow!("основной файл повреждён, а копию не открыть: нет кода доступа на носителе (подключите устройство)")
    })?;
    let key = key_from_token(&token);
    for bak in newest_backups(path) {
        if let Ok(wrapped) = fs::read(&bak) {
            if let Ok(bytes) = decrypt_blob(&wrapped, &key) {
                if let Ok(parsed) = parse_container(&bytes) {
                    let _ = fs::write(path, bytes.as_slice());
                    return Ok(parsed);
                }
            }
        }
    }
    Err(anyhow!("не удалось восстановить хранилище из резервных копий"))
}

pub fn integrity_changed(vault_path: &str, device_store: &DeviceStore) -> bool {
    match (device_store.load_integrity(), file_hash(std::path::Path::new(vault_path))) {
        (Some(stored), Some(current)) => stored != current,
        _ => false,
    }
}

pub fn wipe_account(vault_path: &str, device_store: &DeviceStore) -> Result<()> {
    let _ = device_store.clear();
    let p = std::path::Path::new(vault_path);
    let _ = fs::remove_file(p);
    let _ = fs::remove_file(integrity_file(p));
    let _ = fs::remove_file(format!("{vault_path}.devicekey"));
    let _ = fs::remove_dir_all(local_backup_dir(p));
    let _ = fs::remove_file(token_path_for(p));
    Ok(())
}

pub fn untrust_device(device_store: &DeviceStore) -> Result<()> {
    device_store.clear()
}

fn load_vaultfile(path: &Path) -> Result<VaultFile> {
    let data = fs::read(path).with_context(|| format!("не могу прочитать хранилище: {}", path.display()))?;
    let value: Value = serde_json::from_slice(&data).context("повреждённый файл хранилища")?;
    let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
    if version < VAULT_VERSION as u64 {
        return Err(anyhow!(
            "хранилище старого формата (v{version}). Удали securevault.svault и securevault.svault.devicekey \
             с Рабочего стола и создай заново (это были тестовые данные)."
        ));
    }
    serde_json::from_value(value).context("повреждённый файл хранилища")
}

fn token_path_for(vault_path: &Path) -> PathBuf {
    let name = vault_path
        .file_name()
        .map(|n| format!(".{}.svtoken", n.to_string_lossy()))
        .unwrap_or_else(|| ".sv.svtoken".into());
    match vault_path.parent() {
        Some(par) => par.join(name),
        None => PathBuf::from(name),
    }
}

fn read_backup_token(vault_path: &Path) -> Option<Vec<u8>> {
    let s = fs::read_to_string(token_path_for(vault_path)).ok()?;
    B64.decode(s.trim()).ok()
}

fn load_or_create_backup_token(vault_path: &Path) -> Option<Vec<u8>> {
    if let Some(t) = read_backup_token(vault_path) {
        return Some(t);
    }
    let t = random_bytes(32);
    if fs::write(token_path_for(vault_path), B64.encode(&t)).is_ok() {
        Some(t)
    } else {
        None
    }
}

fn key_from_token(token: &[u8]) -> DualKey {
    let a = sha256_32(&[token, b"\x00"]);
    let b = sha256_32(&[token, b"\x01"]);
    let mut m = [0u8; MASTER_LEN];
    m[..KEY_LEN].copy_from_slice(&a);
    m[KEY_LEN..].copy_from_slice(&b);
    DualKey::from_bytes(&m).expect("ключ из токена")
}

fn local_backup_dir(vault_path: &Path) -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".securevault")))
        .unwrap_or_else(|_| std::env::temp_dir());
    let id = sha256_hex(vault_path.to_string_lossy().as_bytes());
    base.join("SecureVault").join("backups").join(&id[..16])
}

fn write_backup(vault_path: &Path, bytes: &[u8]) {
    let token = match load_or_create_backup_token(vault_path) {
        Some(t) => t,
        None => return,
    };
    let key = key_from_token(&token);
    let wrapped = match encrypt_blob(bytes, &key) {
        Ok(w) => w,
        Err(_) => return,
    };
    let dir = local_backup_dir(vault_path);
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let name = format!("{}.svaultbak", chrono::Utc::now().timestamp_millis());
    let _ = fs::write(dir.join(name), &wrapped);
    if let Ok(rd) = fs::read_dir(&dir) {
        let mut files: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "svaultbak"))
            .collect();
        files.sort();
        while files.len() > 3 {
            let old = files.remove(0);
            let _ = fs::remove_file(old);
        }
    }
}

fn newest_backups(vault_path: &Path) -> Vec<PathBuf> {
    let dir = local_backup_dir(vault_path);
    let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "svaultbak"))
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    files.reverse();
    files
}

fn load_vaultfile_or_recover(path: &Path) -> Result<VaultFile> {
    if let Ok(vf) = load_vaultfile(path) {
        return Ok(vf);
    }
    for bak in newest_backups(path) {
        if let Ok(vf) = load_vaultfile(&bak) {
            let _ = fs::copy(&bak, path);
            return Ok(vf);
        }
    }
    load_vaultfile(path)
}

fn decrypt_index(index_blob: &str, vk: &DualKey) -> Result<Index> {
    let blob = B64.decode(index_blob).context("повреждённый индекс")?;
    let json = decrypt_blob(&blob, vk).context("не удалось расшифровать индекс")?;
    serde_json::from_slice(&json).context("повреждённый JSON индекса")
}

type HmacSha1 = Hmac<Sha1>;

fn base32_encode(data: &[u8]) -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut out = String::new();
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for &b in data {
        buffer = (buffer << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(A[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(A[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }
    out
}

fn base32_decode(s: &str) -> Option<Vec<u8>> {
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    let mut out = Vec::new();
    for c in s.chars() {
        if c.is_whitespace() || c == '=' {
            continue;
        }
        let v = match c.to_ascii_uppercase() {
            ch @ 'A'..='Z' => ch as u32 - 'A' as u32,
            ch @ '2'..='7' => ch as u32 - '2' as u32 + 26,
            _ => return None,
        };
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

fn totp_at(secret: &[u8], counter: u64) -> String {
    let mut mac = <HmacSha1 as Mac>::new_from_slice(secret).expect("hmac key");
    mac.update(&counter.to_be_bytes());
    let hash = mac.finalize().into_bytes();
    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let bin = ((hash[offset] as u32 & 0x7f) << 24)
        | ((hash[offset + 1] as u32) << 16)
        | ((hash[offset + 2] as u32) << 8)
        | (hash[offset + 3] as u32);
    format!("{:06}", bin % 1_000_000)
}

pub fn totp_new_secret_base32() -> String {
    base32_encode(&random_bytes(20))
}

pub fn totp_url(secret_base32: &str, account: &str) -> String {

    let issuer = "H.R.E.N.%20vault";
    format!("otpauth://totp/{issuer}:{account}?secret={secret_base32}&issuer={issuer}&algorithm=SHA1&digits=6&period=30")
}

pub fn totp_verify(secret_base32: &str, code: &str) -> bool {
    let code = code.trim();
    let secret = match base32_decode(secret_base32) {
        Some(s) => s,
        None => return false,
    };
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return false,
    };
    let step = now / 30;
    [step.wrapping_sub(1), step, step + 1]
        .iter()
        .any(|&c| ct_eq(totp_at(&secret, c).as_bytes(), code.as_bytes()))
}

pub fn totp_now(secret_base32: &str) -> Option<(String, u64)> {
    let secret = base32_decode(secret_base32)?;
    if secret.is_empty() {
        return None;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some((totp_at(&secret, now / 30), 30 - (now % 30)))
}

#[cfg(windows)]
pub fn biometric_available() -> bool {
    use windows::Security::Credentials::UI::{UserConsentVerifier, UserConsentVerifierAvailability};
    UserConsentVerifier::CheckAvailabilityAsync()
        .and_then(|op| op.get())
        .map(|a| a == UserConsentVerifierAvailability::Available)
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn biometric_available() -> bool {
    false
}

#[cfg(windows)]
pub fn biometric_verify(reason: &str) -> bool {
    use windows::core::HSTRING;
    use windows::Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier};
    UserConsentVerifier::RequestVerificationAsync(&HSTRING::from(reason))
        .and_then(|op| op.get())
        .map(|r| r == UserConsentVerificationResult::Verified)
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn biometric_verify(_reason: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_roundtrip() {
        let k = DualKey::random();
        let blob = encrypt_blob(b"secret", &k).unwrap();
        assert_eq!(decrypt_blob(&blob, &k).unwrap().as_slice(), b"secret".as_slice());
    }

    #[test]
    fn tampered_blob_fails() {
        let k = DualKey::random();
        let mut blob = encrypt_blob(b"data", &k).unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0x01;
        assert!(decrypt_blob(&blob, &k).is_err());
    }

    #[test]
    fn wrap_needs_both_factors() {
        let vk = DualKey::random();
        let salt = random_bytes(SALT_LEN);
        let ak = random_bytes(ACCOUNT_KEY_LEN);
        let other = random_bytes(ACCOUNT_KEY_LEN);
        let w = wrap_key(&vk, &unlock_kek(b"pw", Some(ak.as_slice()), &salt).unwrap()).unwrap();
        assert!(unwrap_key(&w, &unlock_kek(b"pw", Some(ak.as_slice()), &salt).unwrap()).is_ok());
        assert!(unwrap_key(&w, &unlock_kek(b"pw", Some(other.as_slice()), &salt).unwrap()).is_err());
        assert!(unwrap_key(&w, &unlock_kek(b"bad", Some(ak.as_slice()), &salt).unwrap()).is_err());

        let wb = wrap_key(&vk, &unlock_kek(b"pw", None, &salt).unwrap()).unwrap();
        assert!(unwrap_key(&wb, &unlock_kek(b"pw", None, &salt).unwrap()).is_ok());
        assert!(unwrap_key(&wb, &unlock_kek(b"pw", Some(ak.as_slice()), &salt).unwrap()).is_err());
    }

    #[test]
    fn file_devicestore_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "sv_ds_{}",
            random_bytes(4).iter().map(|b| format!("{:02x}", b)).collect::<String>()
        ));
        fs::create_dir_all(&dir).unwrap();
        let ds = DeviceStore::file(dir.join("dk"));
        assert!(!ds.exists());
        let ak = random_bytes(ACCOUNT_KEY_LEN);
        ds.store(&ak).unwrap();
        assert!(ds.exists());
        assert_eq!(ds.load().unwrap(), ak);
        ds.clear().unwrap();
        assert!(!ds.exists());
    }

    #[test]
    fn v3_container_roundtrip() {
        let dir = std::env::temp_dir().join(format!("sv_v3_{}", hex(&random_bytes(4))));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.svault");
        let mut v = Vault::init(&path, DeviceStore::file(dir.join("dk")), b"pw").unwrap();
        let id = v
            .add_record("note", "t", serde_json::json!({ "content": "hello" }), vec![])
            .unwrap();
        drop(v);

        let raw = fs::read(&path).unwrap();
        assert_eq!(&raw[..4], MAGIC);
        assert!(is_vault_file(&path));
        assert!(!vault_uses_account_key(&path.to_string_lossy()));

        let v2 = Vault::open(&path, DeviceStore::file(dir.join("dk")), b"pw").unwrap();
        let val = v2.get_record(&id).unwrap();
        assert_eq!(val.get("content").unwrap().as_str().unwrap(), "hello");
        drop(v2);

        assert!(Vault::open(&path, DeviceStore::file(dir.join("dk")), b"bad").is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
