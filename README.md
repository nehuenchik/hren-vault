# H.R.E.N. vault™ (beta)

Local, end-to-end encrypted vault for notes and secrets. Your data never leaves your device unencrypted. Written in Rust.

> ⚠️ **Beta.** The cryptography has **not** yet passed an independent audit. Use at your own risk and keep backups.

## Download & Install (Windows)

**➡️ [Download the installer — HREN-vault-setup.exe](https://github.com/nehuenchik/hren-vault/releases/latest)**

1. On the [latest release](https://github.com/nehuenchik/hren-vault/releases/latest) page, open the **Assets** section and click **`HREN-vault-setup.exe`** to download it.
2. Double-click the downloaded file to start the installer.
3. Follow the wizard: choose language → accept the license → install. A desktop shortcut is created and the app starts.

No terminal, no Rust, and no administrator rights are required. Windows only for now.

### "Windows protected your PC" / the download is blocked

H.R.E.N. vault is a new app that isn't **code-signed** yet, so Microsoft Defender SmartScreen doesn't recognize the publisher and shows a warning. The warning is about the **missing signature, not the contents** — the installer is safe. Here is how to get past it.

**If your browser blocks the download itself** (Edge or Chrome may say the file "can't be downloaded securely" or "was blocked"):

- **Edge:** open Downloads (`Ctrl+J`), find the blocked file, click the **⋯** (or the down-arrow) next to it → **Keep**. If prompted again, choose **Show more → Keep anyway**.
- **Chrome:** open Downloads (`Ctrl+J`), click **Keep** on the blocked file (if hidden, expand **⋯ → Keep**).

**When you run the installer** and see the blue **"Windows protected your PC"** screen:

1. Click **More info**.
2. Click **Run anyway**.

The wizard then opens normally.

> Tip: you can also report the file to Microsoft as a false positive from the SmartScreen dialog, but this is optional and takes time to take effect — the steps above are enough to install now.

### Verify the download (optional)

If you want to be sure the file was not tampered with in transit, check its SHA-256 hash in PowerShell and compare it with the checksum published on the release page:

```powershell
Get-FileHash .\HREN-vault-setup.exe -Algorithm SHA256
```

### Why the warning appears (and when it goes away)

Clearing the warning for **everyone** requires a **code-signing certificate**: an EV certificate removes the SmartScreen prompt immediately, while a standard (OV) certificate builds reputation over time. While the project is in beta it ships unsigned, so the steps above are expected. As more people download and run a given build, SmartScreen may also stop warning on its own.

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
