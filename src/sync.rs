
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub trait SyncProvider {

    fn push(&self, id: &str, data: &[u8]) -> Result<()>;

    fn pull(&self, id: &str) -> Result<Option<Vec<u8>>>;

    fn list(&self) -> Result<Vec<String>>;

    fn delete(&self, id: &str) -> Result<()>;
}

pub struct LocalDir {
    root: PathBuf,
}

impl LocalDir {

    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)
            .with_context(|| format!("создать папку синхронизации {root:?}"))?;
        Ok(Self { root })
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.root.join(format!("{}.blob", hex_name(id)))
    }
}

impl SyncProvider for LocalDir {
    fn push(&self, id: &str, data: &[u8]) -> Result<()> {
        let p = self.path_for(id);
        fs::write(&p, data).with_context(|| format!("запись блоба {p:?}"))
    }

    fn pull(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let p = self.path_for(id);
        match fs::read(&p) {
            Ok(d) => Ok(Some(d)),

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("чтение блоба {p:?}")),
        }
    }

    fn list(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        let entries = fs::read_dir(&self.root)
            .with_context(|| format!("чтение папки {:?}", self.root))?;
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if let Some(stem) = name.strip_suffix(".blob") {
                if let Some(id) = unhex_name(stem) {
                    ids.push(id);
                }
            }
        }
        Ok(ids)
    }

    fn delete(&self, id: &str) -> Result<()> {
        let p = self.path_for(id);
        match fs::remove_file(&p) {
            Ok(()) => Ok(()),

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e).with_context(|| format!("удаление блоба {p:?}")),
        }
    }
}

fn hex_name(id: &str) -> String {
    let mut s = String::with_capacity(id.len() * 2);
    for b in id.as_bytes() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn unhex_name(s: &str) -> Option<String> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let raw = s.as_bytes();
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < raw.len() {
        let hi = (raw[i] as char).to_digit(16)?;
        let lo = (raw[i + 1] as char).to_digit(16)?;
        bytes.push((hi * 16 + lo) as u8);
        i += 2;
    }
    String::from_utf8(bytes).ok()
}

pub struct ServerProvider {
    base: String,
    token: String,
}

#[derive(serde::Deserialize)]
struct AuthResp {
    #[allow(dead_code)]
    user_id: String,
    token: String,
}

impl ServerProvider {

    pub fn register(base: &str, email: &str, password: &str) -> Result<Self> {
        Self::auth(base, "register", email, password)
    }

    pub fn login(base: &str, email: &str, password: &str) -> Result<Self> {
        Self::auth(base, "login", email, password)
    }

    fn auth(base: &str, endpoint: &str, email: &str, password: &str) -> Result<Self> {
        let base = Self::normalize_base(base)?;
        let url = format!("{base}/{endpoint}");
        let resp: AuthResp = ureq::post(&url)
            .send_json(serde_json::json!({
                "email": email,
                "auth_hash": Self::auth_hash(password),
            }))
            .map_err(|e| anyhow::anyhow!("сервер ({endpoint}): {e}"))?
            .into_json()
            .context("разбор ответа авторизации")?;
        Ok(Self { base, token: resp.token })
    }

    fn normalize_base(base: &str) -> Result<String> {
        let b = base.trim().trim_end_matches('/');
        if b.is_empty() {
            return Err(anyhow::anyhow!("адрес сервера не указан"));
        }
        if b.starts_with("http://") || b.starts_with("https://") {
            Ok(b.to_string())
        } else {
            Ok(format!("http://{b}"))
        }
    }

    fn auth_hash(password: &str) -> String {
        let mut h = Sha256::new();
        h.update(password.as_bytes());
        B64.encode(h.finalize())
    }

    fn bearer(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn blob_url(&self, id: &str) -> String {
        format!("{}/blobs/{}", self.base, id)
    }
}

impl SyncProvider for ServerProvider {
    fn push(&self, id: &str, data: &[u8]) -> Result<()> {
        ureq::post(&self.blob_url(id))
            .set("Authorization", &self.bearer())
            .set("Content-Type", "application/octet-stream")
            .send_bytes(data)
            .map_err(|e| anyhow::anyhow!("push {id}: {e}"))?;
        Ok(())
    }

    fn pull(&self, id: &str) -> Result<Option<Vec<u8>>> {
        match ureq::get(&self.blob_url(id))
            .set("Authorization", &self.bearer())
            .call()
        {
            Ok(resp) => {
                let mut buf = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut buf)
                    .context("чтение тела ответа")?;
                Ok(Some(buf))
            }

            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("pull {id}: {e}")),
        }
    }

    fn list(&self) -> Result<Vec<String>> {
        let ids: Vec<String> = ureq::get(&format!("{}/blobs", self.base))
            .set("Authorization", &self.bearer())
            .call()
            .map_err(|e| anyhow::anyhow!("list: {e}"))?
            .into_json()
            .context("разбор списка блобов")?;
        Ok(ids)
    }

    fn delete(&self, id: &str) -> Result<()> {
        ureq::delete(&self.blob_url(id))
            .set("Authorization", &self.bearer())
            .call()
            .map_err(|e| anyhow::anyhow!("delete {id}: {e}"))?;
        Ok(())
    }
}

pub const VAULT_BLOB_ID: &str = "vault";

pub fn upload_vault_file(provider: &dyn SyncProvider, vault_path: impl AsRef<Path>) -> Result<()> {
    let vault_path = vault_path.as_ref();
    let bytes = fs::read(vault_path)
        .with_context(|| format!("чтение контейнера {vault_path:?}"))?;
    provider.push(VAULT_BLOB_ID, &bytes)
}

pub fn download_vault_file(provider: &dyn SyncProvider, vault_path: impl AsRef<Path>) -> Result<bool> {
    let vault_path = vault_path.as_ref();
    match provider.pull(VAULT_BLOB_ID)? {
        Some(bytes) => {
            fs::write(vault_path, &bytes)
                .with_context(|| format!("запись контейнера {vault_path:?}"))?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localdir_roundtrip() {

        let dir = std::env::temp_dir().join(format!("svlt_sync_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let p = LocalDir::new(&dir).unwrap();

        assert!(p.list().unwrap().is_empty());
        assert!(p.pull("index").unwrap().is_none());

        p.push("index", b"cipher-index").unwrap();
        p.push("chunk-1", b"cipher-1").unwrap();
        assert_eq!(p.pull("index").unwrap().as_deref(), Some(&b"cipher-index"[..]));
        assert_eq!(p.pull("chunk-1").unwrap().as_deref(), Some(&b"cipher-1"[..]));

        let mut ids = p.list().unwrap();
        ids.sort();
        assert_eq!(ids, vec!["chunk-1".to_string(), "index".to_string()]);

        p.push("index", b"cipher-index-v2").unwrap();
        assert_eq!(p.pull("index").unwrap().as_deref(), Some(&b"cipher-index-v2"[..]));

        p.delete("index").unwrap();
        assert!(p.pull("index").unwrap().is_none());
        p.delete("index").unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn vault_file_sync_roundtrip() {

        let base = std::env::temp_dir().join(format!("svlt_vaultsync_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();

        let store = base.join("store");
        let vault = base.join("local.svault");
        let provider = LocalDir::new(&store).unwrap();

        assert!(!download_vault_file(&provider, &vault).unwrap());

        let original = b"SVLT...cipher-bytes...";
        fs::write(&vault, original).unwrap();
        upload_vault_file(&provider, &vault).unwrap();

        fs::remove_file(&vault).unwrap();
        assert!(download_vault_file(&provider, &vault).unwrap());
        assert_eq!(fs::read(&vault).unwrap(), original);

        let _ = fs::remove_dir_all(&base);
    }
}
