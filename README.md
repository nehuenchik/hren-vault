# H.R.E.N. vault™ (beta)

Local, end‑to‑end encrypted vault for notes and secrets. Your data never leaves your device unencrypted. Written in Rust.

Локальное E2E‑зашифрованное хранилище заметок и секретов. Данные никогда не покидают устройство в открытом виде. Написано на Rust.

> ⚠️ **Beta / Бета.** The cryptography has **not** yet passed an independent audit. Use at your own risk and keep backups.
> Криптография пока **не** проходила независимый аудит. Используйте на свой риск и делайте резервные копии.

---

## English

### What it is
H.R.E.N. vault stores your notes and secrets in a single encrypted file (`*.svault`). Everything is encrypted locally with your password; without it the file is just noise. Optional **Account Key** adds a true second factor and recovery/other‑device access.

### Features
- Double‑layer authenticated encryption (AES‑256‑GCM → ChaCha20‑Poly1305) with zstd compression, keys derived via Argon2id.
- Optional **Account Key** (second factor) with a "trust this device" choice.
- 2FA (TOTP) and Windows Hello biometrics as local gates.
- Notes + secrets, keyword search over notes (keywords stored encrypted), sort & type filter.
- Cloud sync via a folder you already have (OneDrive / Dropbox / Google Drive) — only encrypted blobs are stored there.
- Light / dark theme, interface scale, English / Russian.

### Security model (short)
- The container starts with the magic `SVLT`; everything after the salt is ciphertext.
- Master key = `Argon2id(password ‖ vault_part [‖ Account Key], salt)`.
- The server (future, closed‑source) and cloud folders only ever see encrypted blobs — decryption needs your password (and Account Key).
- Not protected against malware on an already‑unlocked machine, or a weak password + stolen file.

### Build & run (Windows)
1. Install Rust: https://rustup.rs
2. In this folder:
   ```
   cargo build --release
   .\target\release\sv_gui.exe
   ```
   Or double‑click `RUN-BETA.bat` (builds and creates a desktop shortcut).

Prebuilt binaries will be published under **Releases**.

### License
GNU AGPL‑3.0 (see `LICENSE`). The client is open source; future paid services (cross‑device sync, device management, AI search) are separate.

---

## Русский

### Что это
H.R.E.N. vault хранит заметки и секреты в одном зашифрованном файле (`*.svault`). Всё шифруется локально вашим паролем; без него файл — просто шум. Опциональный **Account Key** — настоящий второй фактор и доступ с других устройств / восстановление.

### Возможности
- Двухслойное аутентифицированное шифрование (AES‑256‑GCM → ChaCha20‑Poly1305) + сжатие zstd, ключи через Argon2id.
- Опциональный **Account Key** (второй фактор) с выбором «доверять этому устройству».
- 2FA (TOTP) и биометрия Windows Hello как локальные гейты.
- Заметки и секреты, поиск по ключевым словам заметок (слова хранятся зашифрованно), сортировка и фильтр по типу.
- Синхронизация через вашу же папку облака (OneDrive / Dropbox / Google Drive) — туда попадают только зашифрованные блобы.
- Светлая / тёмная тема, масштаб интерфейса, EN / RU.

### Модель безопасности (кратко)
- Файл начинается с сигнатуры `SVLT`; всё после соли — шифротекст.
- Мастер‑ключ = `Argon2id(пароль ‖ vault_part [‖ Account Key], соль)`.
- Сервер (в будущем, закрытый) и папки облака видят только зашифрованные блобы — для расшифровки нужен пароль (и Account Key).
- Не защищает от малвари на уже разблокированной машине и от слабого пароля при краже файла.

### Сборка и запуск (Windows)
1. Установите Rust: https://rustup.rs
2. В этой папке:
   ```
   cargo build --release
   .\target\release\sv_gui.exe
   ```
   Или двойной клик по `RUN-BETA.bat` (соберёт и создаст ярлык на рабочем столе).

Готовые сборки будут в разделе **Releases**.

### Лицензия
GNU AGPL‑3.0 (см. `LICENSE`). Клиент открыт; будущие платные сервисы (синхронизация между устройствами, менеджмент устройств, ИИ‑поиск) — отдельно.

---

*H.R.E.N.™ — trademark pending / товарный знак в процессе регистрации.*
