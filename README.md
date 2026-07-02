# H.R.E.N. vault™ (beta)

Local, end-to-end encrypted vault for notes and secrets. Your data never leaves your device unencrypted. Written in Rust.

> ⚠️ **Beta.** The cryptography has **not** yet passed an independent audit. Use at your own risk and keep backups.

## What it is

H.R.E.N. vault stores your notes and secrets in a single encrypted file (`*.svault`). Everything is encrypted locally with your password; without it the file is just noise. An optional **Account Key** adds a true second factor and recovery / other-device access.

## Features

- Double-layer authenticated encryption (AES-256-GCM → ChaCha20-Poly1305) with zstd compression; keys derived via Argon2id.
- Optional **Account Key** (second factor) with a "trust this device" choice.
- 2FA (TOTP) and Windows Hello biometrics as local gates.
- Notes + secrets, keyword search over notes (keywords stored encrypted), sort and type filter.
- Cloud sync via a folder you already have (OneDrive / Dropbox / Google Drive) — only encrypted blobs are stored there.
- Light / dark theme, interface scale, English / Russian UI.

## Security model (short)

- The container starts with the magic `SVLT`; everything after the salt is ciphertext.
- Master key = `Argon2id(password ‖ vault_part [‖ Account Key], salt)`.
- The server (future, closed-source) and cloud folders only ever see encrypted blobs — decryption needs your password (and Account Key).
- Not protected against malware on an already-unlocked machine, or a weak password combined with a stolen file.

## Build & run (Windows)

1. Install Rust: https://rustup.rs
2. In this folder:
   ```
   cargo build --release
   .\target\release\sv_gui.exe
   ```

Prebuilt installers are published under **Releases** — end users don't need Rust.

## License

GNU AGPL-3.0 (see `LICENSE`). The client is open source; future paid services (cross-device sync, device management, AI search) are separate.

---

*H.R.E.N.™ — trademark pending.*
