#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32};
use securevault::{DeviceStore, Vault};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(PartialEq, Clone, Copy, Default)]
enum Lang {
    #[default]
    En,
    Ru,
}

fn t(lang: Lang, key: &str) -> String {
    let (en, ru): (&str, &str) = match key {
        "subtitle" => ("Encrypted vault · any storage", "Зашифрованное хранилище · любой носитель"),
        "vault_word" => ("Vault", "Хранилище"),
        "beta_badge" => ("BETA", "БЕТА"),
        "server_dev" => ("Our server — in development", "Наш сервер — в разработке"),
        "appearance" => ("Appearance", "Вид"),
        "theme" => ("Theme", "Тема"),
        "theme_dark" => ("Dark", "Тёмная"),
        "theme_light" => ("Light", "Светлая"),
        "scale" => ("Interface scale", "Масштаб интерфейса"),
        "sort" => ("Sort", "Сортировка"),
        "sort_newest" => ("Newest first", "Сначала новые"),
        "sort_oldest" => ("Oldest first", "Сначала старые"),
        "filter_all" => ("All", "Все"),
        "filter_notes" => ("Notes only", "Только заметки"),
        "filter_secrets" => ("Secrets only", "Только секреты"),
        "search_hint" => ("Search by keyword in notes", "Поиск по словам в заметках"),
        "trust_device" => ("Trust this device (keep the key here)", "Доверять этому устройству (хранить ключ здесь)"),
        "trust_hint" => ("On: no need to enter the key on this device. Off: you'll enter it each login.", "Вкл: не вводить ключ на этом устройстве. Выкл: вводить его при каждом входе."),
        "help_trust" => (
            "Trust this device.\n\nWhen ON, a copy of your Account Key is stored in this computer's secure store (Windows Credential Manager). Then you sign in with just your password — no need to type the key.\n\nWhen OFF, the key is NOT saved here: every login (and any new device) requires you to enter the Account Key by hand. This keeps the key a true second factor — even someone with your password can't open the vault without it.\n\nOn a shared or someone else's computer, keep this OFF so the key isn't left behind. The key still can't be recovered if lost — keep it saved offline.",
            "Доверять этому устройству.\n\nКогда ВКЛ — копия Account Key сохраняется в защищённом хранилище этого компьютера (Windows Credential Manager). Тогда вход только по паролю, ключ вводить не нужно.\n\nКогда ВЫКЛ — ключ здесь НЕ сохраняется: при каждом входе (и на любом новом устройстве) его нужно вводить вручную. Так ключ остаётся настоящим вторым фактором — даже зная пароль, без ключа хранилище не открыть.\n\nНа чужом или общем компьютере держи ВЫКЛ, чтобы ключ не остался на нём. Потерянный ключ восстановить нельзя — храни его офлайн.",
        ),
        "download_sheet" => ("Download recovery sheet", "Скачать аварийный лист"),
        "sheet_saved" => ("Recovery sheet saved (store it offline!)", "Аварийный лист сохранён (храните офлайн!)"),
        "exists" => ("Vault already exists here — choose Open", "Хранилище здесь уже есть — выберите «Открыть»"),
        "create" => ("Create", "Создать"),
        "login" => ("Sign in", "Войти в аккаунт"),
        "back" => ("Back", "Назад"),
        "open_vault" => ("Open vault", "Открыть хранилище"),
        "vault_path" => ("Vault path", "Путь к хранилищу"),
        "browse" => ("Browse…", "Обзор…"),
        "found_vaults" => ("Found vaults", "Найденные хранилища"),
        "rescan" => ("Rescan drives", "Сканировать носители"),
        "login_title" => ("Sign in", "Вход"),
        "enter_password" => ("Enter password", "Введите пароль"),
        "foreign_device" => ("New device — password and Account Key", "Чужое устройство — пароль и Account Key"),
        "not_found" => ("Vault not found — go back and press Create", "Хранилище не найдено — вернитесь и нажмите «Создать»"),
        "password" => ("Password", "Пароль"),
        "create_title" => ("Create vault", "Создание хранилища"),
        "choose_password" => ("Choose a password — that's enough to sign in", "Придумайте пароль — этого достаточно для входа"),
        "repeat_password" => ("Repeat password", "Повтор пароля"),
        "next" => ("Next", "Далее"),
        "settings" => ("Settings", "Настройки"),
        "language" => ("Language", "Язык"),
        "account" => ("Account", "Аккаунт"),
        "change_password" => ("Change password", "Смена пароля"),
        "new_password" => ("New password", "Новый пароль"),
        "save_password" => ("Save password", "Сохранить пароль"),
        "session" => ("Session: trust & auto-lock", "Сессия: доверие и авто-лок"),
        "trust_days" => ("Trust timer (days)", "Таймер доверия (дни)"),
        "auto_lock" => ("Auto-lock (minutes, 0 = off)", "Авто-лок (минуты, 0 = выкл)"),
        "save" => ("Save", "Сохранить"),
        "saved" => ("Saved", "Сохранено"),
        "devices" => ("Devices", "Устройства"),
        "switch_vault" => ("Switch vault", "Сменить хранилище"),
        "unlocked" => ("unlocked", "разблокировано"),
        "search" => ("Search records...", "Поиск по записям..."),
        "type" => ("Type", "Тип"),
        "note" => ("Note", "Заметка"),
        "secret" => ("Secret", "Секрет"),
        "add" => ("Add", "Добавить"),
        "empty" => ("(empty)", "(пусто)"),
        "open" => ("Open", "Открыть"),
        "delete" => ("Delete", "Удалить"),
        "cancel" => ("Cancel", "Отмена"),
        "ak_for_change" => ("Account Key (required to change password)", "Account Key (нужен для смены пароля)"),
        "auto_locked" => ("Auto-locked after inactivity", "Авто-блокировка после простоя"),
        "locked_min" => ("Locked on minimize — sign in again", "Заблокировано при сворачивании — войдите снова"),
        "details" => ("Details", "Подробнее"),
        "change_key" => ("Change key", "Сменить ключ"),
        "change_credentials" => ("Change sign-in credentials", "Сменить реквизиты для входа"),
        "help_password" => (
            "Changes the master password and re-encrypts the vault header with it. The Account Key is required to confirm. A forgotten password with no recovery email means the data cannot be recovered.",
            "Меняет мастер-пароль и перешифровывает им заголовок хранилища. Для подтверждения нужен Account Key. Забытый пароль без почты-восстановления означает, что данные восстановить нельзя.",
        ),
        "help_changekey" => (
            "Re-issues the Account Key; the old one immediately stops working. The key is the second factor needed to sign in on other devices — save the new one offline, it is shown only once.",
            "Перевыпускает Account Key; старый сразу перестаёт работать. Ключ — второй фактор для входа на других устройствах: сохрани новый оффлайн, он показывается один раз.",
        ),
        "ak_managed_hint" => ("The Account Key can be changed under “Change password”.", "Account Key можно сменить в разделе «Смена пароля»."),
        "details_title" => ("Details", "Подробнее"),
        "tap_to_close" => ("Click anywhere to close", "Нажмите в любом месте, чтобы закрыть"),
        "help_changepw" => (
            "Changes the master password and re-encrypts the vault header with it. The Account Key is required to change the password. You can also re-issue the Account Key here. Keep in mind that a forgotten password with no recovery email means the data cannot be recovered.",
            "Меняет мастер-пароль и перешифровывает им заголовок хранилища. Для смены пароля нужно ввести Account Key. Здесь же можно перевыпустить Account Key. Учтите: забытый пароль без почты-восстановления означает, что данные восстановить нельзя.",
        ),
        "help_ak" => (
            "The Account Key is a second factor. On another device signing in needs both the password and this key; on this device it is trusted, so the password is enough. The key is shown only once, so store it offline and separately from the password. If both the key and the password are lost, access from a new device cannot be restored.",
            "Account Key — второй фактор. На другом устройстве для входа нужны и пароль, и этот ключ; на этом устройстве оно доверенное, поэтому хватает пароля. Ключ показывается только один раз — храните его оффлайн и отдельно от пароля. Если потеряны и ключ, и пароль, доступ с нового устройства восстановить нельзя.",
        ),
        "help_totp" => (
            "Adds one-time codes (TOTP) from an authenticator app, asked after the password. Re-binding generates a new secret and makes the old code invalid. If you lose the authenticator without a saved copy of the secret, sign-in on untrusted devices may become impossible.",
            "Добавляет одноразовые коды (TOTP) из приложения-аутентификатора, которые спрашиваются после пароля. Перепривязка создаёт новый секрет и делает старый код недействительным. Если потерять аутентификатор без сохранённой копии секрета, вход на недоверенных устройствах может стать невозможным.",
        ),
        "help_bio" => (
            "A local Windows Hello check (face, fingerprint or PIN) shown after unlock. Biometrics never leave the device and this is only a convenience gate, not the encryption itself. If Windows Hello stops working, you sign in with the password as usual.",
            "Локальная проверка Windows Hello (лицо, отпечаток или PIN) после входа. Биометрия не покидает устройство, и это лишь гейт удобства, а не само шифрование. Если Windows Hello перестанет работать, вход выполняется паролем как обычно.",
        ),
        "help_session" => (
            "The trust timer marks the device untrusted after the chosen number of days without a password sign-in; then the key and 2FA are required again. Auto-lock closes the vault after the set idle minutes. Changing these settings is confirmed with the Account Key and 2FA.",
            "Таймер доверия помечает устройство недоверенным после выбранного числа дней без входа по паролю — тогда снова потребуются ключ и 2FA. Авто-лок закрывает хранилище после заданного простоя в минутах. Изменение этих настроек подтверждается Account Key и 2FA.",
        ),
        "help_account" => (
            "Sign out closes the vault; the next sign-in on a new device needs the password and the Account Key. Full deletion erases the vault, all records, backups, the device key and 2FA, and this cannot be undone — nothing can restore the data afterwards.",
            "Выход закрывает хранилище; следующий вход на новом устройстве потребует пароль и Account Key. Полное удаление стирает хранилище, все записи, копии, ключ устройства и 2FA — это необратимо, после него данные не вернуть ничем.",
        ),
        "note_desc_hint" => ("Description (optional)", "Описание (необязательно)"),
        "note_content_hint" => ("Note text", "Текст заметки"),
        "secret_desc_hint" => ("Description (required)", "Описание (обязательно)"),
        "secret_content_hint" => ("Secret content", "Содержимое секрета"),
        "secret_note" => ("Secret: no connector, AI can't see it; open with password", "Секрет: без коннектора, ИИ не видит; открыть — по паролю"),
        "list_secret_sub" => ("secret · this device only", "секрет · только с устройства"),
        "list_note" => ("note", "заметка"),
        "wrong_password" => ("Wrong password", "Неверный пароль"),
        "wrong_password_or_ak" => ("Wrong password or Account Key", "Неверный пароль или Account Key"),
        "fill_note" => ("Fill in the note text", "Заполни текст заметки"),
        "secret_need_desc" => ("A secret needs a description (what's inside)", "У секрета обязательно описание (что внутри)"),
        "fill_secret" => ("Fill in the secret content", "Заполни содержимое секрета"),
        "desc_ne_content" => ("Description must differ from content", "Описание не должно совпадать с содержимым"),
        "pwd_empty" => ("Password can't be empty", "Пароль не может быть пустым"),
        "pwd_mismatch" => ("Passwords don't match", "Пароли не совпадают"),
        "record_added" => ("Record added", "Запись добавлена"),
        "reg_key_title" => ("Account Key", "Account Key"),
        "reg_key_intro" => ("An Account Key is required: it protects access from other devices. Generate and save it now.", "Account Key обязателен: он защищает доступ с других устройств. Сгенерируй и сохрани его сейчас."),
        "entropy_hint" => ("Move the mouse to fill the bar — its path becomes the key's noise", "Поводите мышью, заполняя шкалу — её траектория станет шумом ключа"),
        "entropy_move" => ("keep moving the mouse...", "двигайте мышью..."),
        "reg_key_create" => ("Create Account Key", "Создать Account Key"),
        "reg_key_skip" => ("Skip (password only)", "Пропустить (только пароль)"),
        "reg_key_save_warn" => ("Save this key offline (paper), separately from the password. Without it and the password, access can't be recovered. Paste it below to continue.", "Сохрани ключ оффлайн (бумага), отдельно от пароля. Без него и пароля доступ не вернуть. Вставь его ниже, чтобы продолжить."),
        "reg_key_paste" => ("Paste the Account Key to confirm", "Вставь Account Key для подтверждения"),
        "reg_key_continue" => ("Continue", "Продолжить"),
        "copy" => ("Copy", "Копировать"),
        "copied" => ("Copied", "Скопировано"),
        "settings_saved" => ("Session settings saved", "Настройки сессии сохранены"),
        "password_changed" => ("Password changed", "Пароль изменён"),
        "record_deleted" => ("Record deleted", "Запись удалена"),
        "pwd_pair_bad" => ("Passwords are empty or don't match", "Пароли пусты или не совпадают"),
        "session_warn" => ("Changing session settings requires Account Key and 2FA", "Смена настроек сессии требует Account Key и 2FA"),
        "totp_code" => ("2FA code", "Код 2FA"),
        "wrong_2fa" => ("Wrong 2FA code", "Неверный код 2FA"),
        "wrong_ak" => ("Wrong Account Key", "Неверный Account Key"),
        "done" => ("Done", "Готово"),
        "confirm" => ("Confirm", "Подтвердить"),
        "exit" => ("Exit", "Выйти"),
        "got_it" => ("Got it", "Понятно"),
        "twofa_check" => ("Two-factor check", "Двухфакторная проверка"),
        "enter_code" => ("Enter the code from your authenticator app", "Введите код из приложения-аутентификатора"),
        "biometrics" => ("Biometrics", "Биометрия"),
        "bio_confirm_login" => ("Confirm sign-in via Windows Hello", "Подтвердите вход через Windows Hello"),
        "bio_waiting" => ("Waiting for Windows Hello...", "Ожидание Windows Hello..."),
        "bio_reason_login" => ("Sign in to H.R.E.N. vault", "Вход в H.R.E.N. vault"),
        "bio_reason_enable" => ("Confirm to enable biometrics", "Подтвердите для включения биометрии"),
        "integrity_warn" => ("WARNING: vault file changed outside the app (possible tamper, rollback or corruption). Check your security.", "ВНИМАНИЕ: файл хранилища изменён вне приложения (возможна подмена, откат или повреждение). Проверьте безопасность."),
        "open_secret_confirm" => ("Open secret — confirm with password", "Открытие секрета — подтвердите паролем"),
        "delete_record_confirm" => ("Delete record — confirm with password", "Удаление записи — подтвердите паролем"),
        "delete_cancelled" => ("Wrong password — deletion cancelled", "Неверный пароль — удаление отменено"),
        "content" => ("Content", "Содержимое"),
        "close" => ("Close", "Закрыть"),
        "collapse" => ("Collapse", "Свернуть"),
        "expand" => ("Expand", "Развернуть"),
        "totp_rebind" => ("Re-bind (new secret)", "Перепривязать (новый секрет)"),
        "totp_disable" => ("Disable 2FA", "Отключить 2FA"),
        "disable_2fa_confirm" => ("Disabling 2FA — confirm with password:", "Отключение 2FA — подтвердите паролем:"),
        "confirm_disable" => ("Confirm disable", "Подтвердить отключение"),
        "rebind_warn" => ("Re-binding invalidates the old secret. Confirm with password:", "Перепривязка сделает старый секрет недействительным. Подтвердите паролем:"),
        "twofa_disabled" => ("2FA disabled", "2FA отключена"),
        "secret_reissued" => ("Secret reissued — scan the new QR", "Секрет перевыпущен — отсканируй новый QR"),
        "totp_scan" => ("Scan in Authy / Google Authenticator:", "Отсканируй в Authy / Google Authenticator:"),
        "totp_manual" => ("or enter the secret manually:", "или введи секрет вручную:"),
        "bio_desc1" => ("Uses built-in Windows biometrics. What triggers — face, fingerprint or PIN — depends on your system setup.", "Использует встроенную биометрию Windows. Что именно сработает — лицо, отпечаток или PIN — зависит от того, что настроено в системе."),
        "bio_desc2" => ("Customize: Windows Settings > Accounts > Sign-in options.", "Настроить под себя: Параметры Windows > Учётные записи > Варианты входа."),
        "bio_disable" => ("Disable biometrics", "Отключить биометрию"),
        "bio_disable_confirm" => ("Disabling biometrics — confirm with password:", "Отключение биометрии — подтвердите паролем:"),
        "bio_disabled" => ("Biometrics disabled", "Биометрия отключена"),
        "bio_unavail" => ("Unavailable on this device (no Windows Hello)", "Недоступна на этом устройстве (нет Windows Hello)"),
        "bio_enabled_msg" => ("Biometrics enabled", "Биометрия включена"),
        "bio_not_confirmed" => ("Biometrics not confirmed", "Биометрия не подтверждена"),
        "ak_desc" => ("Optional hardening. Enable it and signing in on ANOTHER device needs BOTH the password AND this key; this device becomes trusted (password is enough).", "Опциональное усиление. Включишь — для входа на ДРУГОМ устройстве понадобятся И пароль, И этот ключ; текущее устройство станет доверенным (хватит пароля)."),
        "ak_rotate" => ("Update Account Key", "Обновить Account Key"),
        "ak_delete" => ("Delete Account Key", "Удалить Account Key"),
        "ak_rotate_warn" => ("Re-issue: the old Account Key stops working. Confirm with password:", "Перевыпуск: старый Account Key перестанет действовать. Подтвердите паролем:"),
        "ak_reissue_btn" => ("Re-issue", "Перевыпустить"),
        "ak_reissued" => ("Account Key re-issued", "Account Key перевыпущен"),
        "ak_delete_warn" => ("Deleting returns to password-only sign-in and removes device trust. Confirm with ALL factors:", "Удаление вернёт вход только по паролю и снимет доверие устройств. Подтвердите ВСЕМИ факторами:"),
        "ak_current" => ("Current Account Key", "Текущий Account Key"),
        "ak_disabled_msg" => ("Account Key disabled — sign in with password now", "Account Key отключён — вход теперь по паролю"),
        "ak_enable_warn" => ("After enabling, save the shown key offline. Without it and the password, access from a new device can't be recovered. Confirm with password:", "После включения сохрани показанный ключ оффлайн. Без него и пароля доступ с нового устройства не вернуть. Подтвердите паролем:"),
        "enable_btn" => ("Enable", "Включить"),
        "ak_enabled_msg" => ("Account Key enabled — this device is trusted", "Account Key включён — это устройство доверенное"),
        "ak_save_once" => ("SAVE THIS Account Key (shown once):", "СОХРАНИ ЭТОТ Account Key (показывается один раз):"),
        "wipe_title" => ("FULL ACCOUNT DELETION", "ПОЛНОЕ УДАЛЕНИЕ АККАУНТА"),
        "wipe_warn" => ("Irreversible: the vault, all records, backups, device key and 2FA will be erased. Nothing can restore it.", "Необратимо: будут стёрты хранилище, все записи, резервные копии, ключ устройства и 2FA. Восстановить нельзя ничем."),
        "confirm_all_factors" => ("Confirm with all factors of your level:", "Подтвердите всеми факторами уровня:"),
        "wipe_btn" => ("Delete forever", "Удалить навсегда"),
        "account_wiped" => ("Account fully deleted.", "Аккаунт полностью удалён."),
        "signout_warn_ak" => ("Signing out removes trust from this device: next sign-in needs BOTH the password AND the Account Key. Make sure the Account Key is saved.", "Выход снимет доверие с этого устройства: при следующем входе понадобятся И пароль, И Account Key. Убедись, что Account Key сохранён."),
        "signout_warn_base" => ("Sign out (the vault will close). Sign in with a password.", "Выйти из аккаунта (хранилище закроется). Вход — по паролю."),
        "signout_yes" => ("Yes, sign out", "Да, выйти из аккаунта"),
        "signed_out_ak" => ("Signed out. Sign in needs password and Account Key.", "Вы вышли из аккаунта. Для входа нужны пароль и Account Key."),
        "signed_out_base" => ("Signed out. Sign in needs a password.", "Вы вышли из аккаунта. Для входа нужен пароль."),
        "vault_not_found" => ("Vault not found", "Хранилище не найдено"),
        "deleted_record" => ("Record deleted", "Запись удалена"),
        "created_at" => ("created", "создано"),
        "loading" => ("Loading…", "Загрузка…"),
        "totp_title" => ("Two-factor protection (TOTP)", "Двухфакторная защита (TOTP)"),
        "totp_enable" => ("Enable 2FA", "Включить 2FA"),
        "on" => ("Enabled", "Включено"),
        "bio_title" => ("Biometrics (Windows Hello)", "Биометрия (Windows Hello)"),
        "bio_enable" => ("Enable biometrics", "Включить биометрию"),
        "ak_title" => ("Account Key (trusted devices)", "Account Key (доверенные устройства)"),
        "ak_enable" => ("Enable Account Key", "Включить Account Key"),
        "ak_enable_hint" => ("Account Key is off in this vault. Enter your password to enable it.", "В этом хранилище Account Key выключен. Введите пароль, чтобы включить его."),
        "ak_enabled_ok" => ("Account Key enabled — save it below", "Account Key включён — сохраните его ниже"),
        "sign_out" => ("Sign out", "Выйти из аккаунта"),
        "delete_account" => ("Delete account permanently", "Удалить аккаунт полностью"),
        "sync" => ("Sync", "Синхронизация"),
        "sync_mode_server" => ("Our server", "Наш сервер"),
        "sync_mode_cloud" => ("Your cloud (folder)", "Своё облако (папка)"),
        "sync_cloud_dir" => ("Cloud folder", "Папка облака"),
        "sync_pick_folder" => ("Choose…", "Выбрать…"),
        "sync_cloud_hint" => (
            "Pick a folder inside OneDrive/Dropbox/Google Drive. The vault is saved there encrypted, and your cloud app uploads it automatically. No server, no account — the cloud only ever sees encrypted blobs.",
            "Выберите папку внутри OneDrive/Dropbox/Google Drive. Хранилище сохранится туда зашифрованным, а клиент облака сам зальёт его. Без сервера и аккаунта — облако видит только зашифрованные блобы.",
        ),
        "sync_url" => ("Server address", "Адрес сервера"),
        "sync_email" => ("Email", "Почта"),
        "sync_register" => ("Register", "Регистрация"),
        "sync_login" => ("Sign in", "Вход"),
        "sync_upload" => ("Upload vault", "Залить хранилище"),
        "sync_download" => ("Download vault", "Скачать хранилище"),
        "sync_connected" => ("Connected to server", "Подключено к серверу"),
        "sync_not_connected" => ("Not connected. Sign in or register first.", "Нет подключения. Сначала вход или регистрация."),
        "sync_uploaded" => ("Vault uploaded to server", "Хранилище залито на сервер"),
        "sync_downloaded" => ("Vault downloaded — reopen it", "Хранилище скачано — откройте его заново"),
        "sync_empty" => ("Nothing on server yet", "На сервере пока пусто"),
        "sync_download_warn" => ("Download overwrites the local file with the server copy.", "Скачивание перезапишет локальный файл копией с сервера."),
        "help_sync" => (
            "Two ways to sync, both store only encrypted blobs. 'Our server' — sign in with email+password (separate from your vault password) to upload your encrypted file. 'Your cloud (folder)' — pick a folder in OneDrive/Dropbox/Drive; no server or account needed. Either way, decryption still needs your vault password (and Account Key).",
            "Два способа синхронизации, оба хранят только зашифрованные блобы. «Наш сервер» — вход по почте+паролю (отдельный от пароля хранилища), заливает ваш зашифрованный файл. «Своё облако (папка)» — выберите папку в OneDrive/Dropbox/Drive; сервер и аккаунт не нужны. В любом случае для расшифровки нужен пароль хранилища (и Account Key).",
        ),
        _ => (key, key),
    };
    match lang {
        Lang::En => en.to_string(),
        Lang::Ru => ru.to_string(),
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    language: String,
    last_vault: String,
    recent_vaults: Vec<String>,
    #[serde(default)]
    sync_url: String,
    #[serde(default)]
    sync_email: String,
    #[serde(default)]
    sync_mode: String,
    #[serde(default)]
    sync_cloud_dir: String,
    #[serde(default)]
    theme: String,
    #[serde(default = "default_scale")]
    ui_scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

fn config_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("SecureVault").join("config.json")
}

fn load_config() -> Config {
    std::fs::read(config_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &Config) {
    let p = config_path();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_vec_pretty(cfg) {
        let _ = std::fs::write(p, json);
    }
}

#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    card: Color32,
    card2: Color32,
    accent: Color32,
    accent_dim: Color32,
    green: Color32,
    text: Color32,
    muted: Color32,
    danger: Color32,
    btn: Color32,
    btn_stroke: Color32,
    hover_bg: Color32,
    extreme: Color32,
}

const DARK: Palette = Palette {
    bg: Color32::from_rgb(13, 17, 23),
    card: Color32::from_rgb(22, 27, 34),
    card2: Color32::from_rgb(28, 34, 43),
    accent: Color32::from_rgb(0, 224, 198),
    accent_dim: Color32::from_rgb(0, 150, 136),
    green: Color32::from_rgb(63, 224, 150),
    text: Color32::from_rgb(220, 234, 240),
    muted: Color32::from_rgb(120, 140, 150),
    danger: Color32::from_rgb(255, 95, 95),
    btn: Color32::from_rgb(40, 50, 62),
    btn_stroke: Color32::from_rgb(72, 88, 104),
    hover_bg: Color32::from_rgb(26, 40, 46),
    extreme: Color32::from_rgb(10, 13, 18),
};

const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(208, 209, 206),
    card: Color32::from_rgb(228, 229, 226),
    card2: Color32::from_rgb(198, 200, 197),
    accent: Color32::from_rgb(0, 122, 109),
    accent_dim: Color32::from_rgb(0, 100, 90),
    green: Color32::from_rgb(26, 112, 68),
    text: Color32::from_rgb(26, 28, 28),
    muted: Color32::from_rgb(84, 88, 86),
    danger: Color32::from_rgb(178, 42, 38),
    btn: Color32::from_rgb(216, 217, 214),
    btn_stroke: Color32::from_rgb(176, 178, 174),
    hover_bg: Color32::from_rgb(206, 216, 210),
    extreme: Color32::from_rgb(232, 233, 230),
};

thread_local! {
    static PAL: std::cell::Cell<Palette> = const { std::cell::Cell::new(DARK) };
}

fn pal() -> Palette {
    PAL.with(|p| p.get())
}

fn set_palette(light: bool) {
    PAL.with(|p| p.set(if light { LIGHT } else { DARK }));
}

fn default_vault_path() -> String {
    if let Ok(up) = std::env::var("USERPROFILE") {
        format!("{up}\\Desktop\\securevault.svault")
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{home}/Desktop/securevault.svault")
    } else {
        "securevault.svault".into()
    }
}

fn device_store(vault: &str) -> DeviceStore {
    DeviceStore::keychain(vault.to_string(), format!("{vault}.devicekey"))
}

fn key_pressed(ui: &egui::Ui, key: egui::Key) -> bool {
    ui.input(|i| i.key_pressed(key))
}

fn draw_qr(ui: &mut egui::Ui, data: &str) {
    let code = match qrcode::QrCode::new(data.as_bytes()) {
        Ok(c) => c,
        Err(_) => {
            ui.label("QR недоступен");
            return;
        }
    };
    let w = code.width();
    let colors = code.to_colors();
    let scale = 4.0_f32;
    let margin = 4.0_f32;
    let total = (w as f32 + margin * 2.0) * scale;
    let (resp, painter) = ui.allocate_painter(egui::vec2(total, total), egui::Sense::hover());
    let origin = resp.rect.min;
    painter.rect_filled(resp.rect, 0.0, Color32::WHITE);
    for y in 0..w {
        for x in 0..w {
            if colors[y * w + x] == qrcode::Color::Dark {
                let p = origin + egui::vec2((x as f32 + margin) * scale, (y as f32 + margin) * scale);
                painter.rect_filled(egui::Rect::from_min_size(p, egui::vec2(scale, scale)), 0.0, Color32::BLACK);
            }
        }
    }
}

fn connectors(text: &str) -> Vec<String> {
    let stop = ["и", "в", "на", "с", "по", "за", "для", "от", "до", "the", "and", "for"];
    let mut out: Vec<String> = Vec::new();
    for w in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if w.chars().count() >= 3 && !stop.contains(&w) && !out.iter().any(|x| x == w) {
            out.push(w.to_string());
        }
        if out.len() >= 40 {
            break;
        }
    }
    out
}

fn auto_describe(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or("").trim();
    let base = if first_line.is_empty() { text.trim() } else { first_line };
    let short: String = base.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
    let short: String = short.chars().take(45).collect();
    if short.is_empty() { "Заметка".to_string() } else { short }
}

fn content_text(val: &serde_json::Value) -> String {
    val.get("content")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| serde_json::to_string_pretty(val).unwrap_or_default())
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum NewType {
    #[default]
    Note,
    Secret,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum BioAction {
    #[default]
    None,
    Enable,
    Unlock,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Focus {
    #[default]
    None,
    AddDesc,
    AddContent,
    DelPass,
    OpenPass,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Screen {
    #[default]
    Splash,
    Start,
    Login,
    RegPass,
    RegKey,
    Main,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum SettingsSection {
    #[default]
    None,
    Language,
    Display,
    ChangePw,
    Totp,
    Bio,
    Session,
    Account,
}

#[derive(Default)]
struct App {
    vault_path: String,
    screen: Screen,
    error: String,
    info: String,
    themed: bool,
    need_focus: bool,
    vault: Option<Vault>,
    password: String,
    account_key: String,
    login_trust: bool,
    reg_pass: String,
    reg_pass2: String,
    selected_content: String,
    selected_meta: String,
    selected_id: Option<String>,
    selected_orig: String,
    confirm_signout: bool,
    integrity_warn: bool,
    integrity_ack: bool,
    awaiting_totp: bool,
    totp_code: String,
    totp_setup: String,
    rebind_confirm: bool,
    rebind_password: String,
    confirm_delete: bool,
    del_pw: String,
    del_ak: String,
    del_totp: String,
    disable_2fa_confirm: bool,
    dis_pw: String,
    awaiting_bio: bool,
    disable_bio_confirm: bool,
    bio_pw: String,
    bio_available: bool,
    bio_rx: Option<std::sync::mpsc::Receiver<bool>>,
    bio_action: BioAction,
    new_desc: String,
    new_content: String,
    new_type: NewType,
    want_focus: Focus,
    show_settings: bool,
    np1: String,
    np2: String,
    pending_delete: Option<String>,
    del_password: String,
    pending_open: Option<String>,
    open_password: String,

    ak_rotate_pw: String,
    ak_show: String,

    lang: Lang,
    config: Config,
    np_ak: String,
    sess_trust_days: u32,
    sess_lock_min: String,
    last_active: Option<std::time::Instant>,
    reg_key_confirm: String,
    reg_trust: bool,
    reg_entropy: Vec<u8>,
    reg_last_pos: Option<egui::Pos2>,
    reg_move_dist: f32,
    open_section: SettingsSection,
    help_key: &'static str,
    help_armed: &'static str,
    cred_open: u8,
    splash_start: Option<std::time::Instant>,
    sess_ak: String,
    sess_totp: String,

    applied_scale: f32,
    sort_oldest: bool,
    popup_armed: bool,
    query: String,
    filter_kind: u8,
}

impl App {
    fn new() -> Self {
        let mut config = load_config();

        if !(0.5..=3.0).contains(&config.ui_scale) {
            config.ui_scale = 1.0;
        }
        set_palette(config.theme == "light");
        let lang = if config.language == "ru" { Lang::Ru } else { Lang::En };
        let vault_path = if !config.last_vault.is_empty() {
            config.last_vault.clone()
        } else {
            default_vault_path()
        };
        App {
            vault_path,
            lang,
            config,
            bio_available: securevault::biometric_available(),
            splash_start: Some(std::time::Instant::now()),
            reg_trust: true,
            ..Default::default()
        }
    }

    fn tr(&self, key: &str) -> String {
        t(self.lang, key)
    }

    fn current_help(&self) -> Option<&'static str> {

        if self.help_key.is_empty() {
            None
        } else {
            Some(self.help_key)
        }
    }

    fn section(&mut self, ui: &mut egui::Ui, sec: SettingsSection, title: &str) -> bool {
        let open = self.open_section == sec;
        let arrow = if open { "-" } else { "+" };
        ui.add_space(6.0);
        if ui
            .add_sized([ui.available_width(), 30.0], egui::Button::new(format!("{arrow}  {title}")))
            .clicked()
        {
            self.open_section = if open { SettingsSection::None } else { sec };

            self.confirm_delete = false;
            self.confirm_signout = false;
            self.cred_open = 0;
            self.disable_2fa_confirm = false;
            self.rebind_confirm = false;
            self.disable_bio_confirm = false;
            self.totp_setup.clear();
            self.error.clear();
        }
        self.open_section == sec
    }

    fn localize_err(&self, e: &str) -> String {
        if e.contains("Account Key") && e.contains("Неверный пароль") {
            self.tr("wrong_password_or_ak")
        } else if e.contains("Неверный пароль") {
            self.tr("wrong_password")
        } else {
            e.to_string()
        }
    }

    fn remember_vault(&mut self) {
        self.config.last_vault = self.vault_path.clone();
        self.config.recent_vaults.retain(|p| p != &self.vault_path);
        self.config.recent_vaults.insert(0, self.vault_path.clone());
        self.config.recent_vaults.truncate(8);
        save_config(&self.config);
    }

    fn touch(&mut self) {
        self.last_active = Some(std::time::Instant::now());
    }

    fn has_biometric(&self) -> bool {
        self.vault
            .as_ref()
            .is_some_and(|v| v.records().iter().any(|r| r.rec_type == "biometric"))
    }

    fn start_bio(&mut self, action: BioAction, reason: &str) {
        if self.bio_rx.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let reason = reason.to_string();
        std::thread::spawn(move || {
            let _ = tx.send(securevault::biometric_verify(&reason));
        });
        self.bio_rx = Some(rx);
        self.bio_action = action;
        self.error.clear();
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.themed {
            return;
        }
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(30.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(17.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(15.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
        ]
        .into();

        style.spacing.button_padding = egui::vec2(18.0, 12.0);
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.interact_size.y = 40.0;

        let r = egui::Rounding::same(10.0);
        let p = pal();
        let light = self.config.theme == "light";

        let mut v = if light { egui::Visuals::light() } else { egui::Visuals::dark() };
        v.panel_fill = p.bg;
        v.window_fill = p.bg;
        v.window_rounding = egui::Rounding::same(12.0);
        v.extreme_bg_color = p.extreme;
        v.override_text_color = Some(p.text);

        v.widgets.inactive.rounding = r;
        v.widgets.inactive.bg_fill = p.btn;
        v.widgets.inactive.weak_bg_fill = p.btn;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, p.btn_stroke);
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, p.text);

        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.text);

        v.widgets.hovered.rounding = r;
        v.widgets.hovered.bg_fill = p.hover_bg;
        v.widgets.hovered.weak_bg_fill = p.hover_bg;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, p.accent);
        v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, p.accent);
        v.widgets.hovered.expansion = 1.0;

        v.widgets.active.rounding = r;
        v.widgets.active.bg_fill = p.accent_dim;
        v.widgets.active.weak_bg_fill = p.accent_dim;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.5, p.accent);
        v.widgets.active.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
        v.selection.bg_fill = p.accent_dim;
        v.selection.stroke = egui::Stroke::new(1.0, p.accent);

        style.visuals = v;
        ctx.set_style(style);
        self.themed = true;
    }

    fn focus(&mut self, want: Focus, resp: &egui::Response) {
        if self.want_focus == want {
            resp.request_focus();
            self.want_focus = Focus::None;
        }
    }

    fn vault_exists(&self) -> bool {
        std::path::Path::new(&self.vault_path).exists()
    }

    fn ak_enabled(&self) -> bool {
        self.vault.as_ref().is_some_and(|v| v.account_key_enabled())
    }

    fn has_totp(&self) -> bool {
        self.vault
            .as_ref()
            .is_some_and(|v| v.records().iter().any(|r| r.rec_type == "totp_secret"))
    }

    fn totp_secret(&self) -> Option<String> {
        let v = self.vault.as_ref()?;
        let rec = v.records().iter().find(|r| r.rec_type == "totp_secret")?;
        let val = v.get_record(&rec.id).ok()?;
        val.get("secret").and_then(|s| s.as_str()).map(|s| s.to_string())
    }

    fn splash_ui(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_height();
        ui.add_space((avail * 0.32).max(40.0));
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("H.R.E.N.™").size(52.0).strong().color(pal().accent));
            ui.add_space(2.0);
            ui.label(egui::RichText::new(self.tr("vault_word")).size(22.0).strong().color(pal().muted));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(self.tr("beta_badge")).size(22.0).strong().color(pal().green));
            ui.add_space(26.0);
            ui.add(egui::Spinner::new().size(36.0).color(pal().accent));
            ui.add_space(12.0);
            ui.label(egui::RichText::new(self.tr("loading")).size(13.0).color(pal().muted));
        });
    }

    fn start_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("H.R.E.N.™").size(48.0).strong().color(pal().accent));
            ui.add_space(2.0);
            ui.label(egui::RichText::new(self.tr("vault_word")).size(20.0).strong().color(pal().muted));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(self.tr("beta_badge")).size(20.0).strong().color(pal().green));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(self.tr("subtitle")).size(12.0).color(pal().muted));
        });
        ui.add_space(18.0);

        ui.horizontal(|ui| {
            ui.label(self.tr("language"));
            if ui.selectable_label(self.lang == Lang::En, "EN").clicked() {
                self.lang = Lang::En;
                self.config.language = "en".into();
                save_config(&self.config);
            }
            if ui.selectable_label(self.lang == Lang::Ru, "RU").clicked() {
                self.lang = Lang::Ru;
                self.config.language = "ru".into();
                save_config(&self.config);
            }
        });
        ui.add_space(10.0);

        ui.label(self.tr("vault_path"));
        let browse_lbl = self.tr("browse");
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 100.0).max(120.0);
            ui.add(egui::TextEdit::singleline(&mut self.vault_path).desired_width(w));
            if ui.button(browse_lbl).clicked() {
                let mut d = rfd::FileDialog::new()
                    .add_filter("SecureVault", &["svault"])
                    .set_file_name("securevault.svault");
                if let Some(parent) = std::path::Path::new(&self.vault_path).parent() {
                    if parent.is_dir() {
                        d = d.set_directory(parent);
                    }
                }
                if let Some(p) = d.save_file() {
                    self.vault_path = p.display().to_string();
                    self.error.clear();
                }
            }
        });
        ui.add_space(14.0);
        ui.vertical_centered(|ui| {
            if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new(self.tr("create")).size(18.0).strong())).clicked() {
                if self.vault_exists() {
                    self.error = self.tr("exists");
                } else {
                    self.error.clear();
                    self.reg_pass.clear();
                    self.reg_pass2.clear();
                    self.need_focus = true;
                    self.screen = Screen::RegPass;
                }
            }
            ui.add_space(10.0);
            if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new(self.tr("open_vault")).size(18.0))).clicked() {
                self.error.clear();
                self.password.clear();
                self.account_key.clear();
                self.need_focus = true;
                self.screen = Screen::Login;
            }
        });
        ui.add_space(10.0);
        if !self.error.is_empty() {
            ui.add_space(10.0);
            ui.label(egui::RichText::new(&self.error).color(pal().danger));
        }
        if !self.info.is_empty() {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(&self.info).color(pal().green));
        }
    }

    fn login_ui(&mut self, ui: &mut egui::Ui) {
        let exists = self.vault_exists();
        let uses_ak = exists && securevault::vault_uses_account_key(&self.vault_path);
        let trusted = Vault::is_trusted(&device_store(&self.vault_path));
        let need_ak = uses_ak && !trusted;
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(self.tr("login_title")).size(22.0).strong().color(pal().accent));
            ui.add_space(14.0);
            if !exists {
                ui.label(egui::RichText::new(self.tr("not_found")).color(pal().danger));
            } else if need_ak {
                ui.label(egui::RichText::new(self.tr("foreign_device")).color(pal().muted));
            } else {
                ui.label(egui::RichText::new(self.tr("enter_password")).color(pal().muted));
            }
            ui.add_space(12.0);
            let pw_hint = t(self.lang, "password");
            let pw = ui.add_sized([280.0, 32.0], egui::TextEdit::singleline(&mut self.password).password(true).hint_text(pw_hint));
            if self.need_focus {
                pw.request_focus();
                self.need_focus = false;
            }
            let mut submit = pw.lost_focus() && key_pressed(ui, egui::Key::Enter);
            if need_ak {
                ui.add_space(8.0);
                let ak = ui.add_sized([280.0, 32.0], egui::TextEdit::singleline(&mut self.account_key).hint_text("Account Key"));
                submit |= ak.lost_focus() && key_pressed(ui, egui::Key::Enter);
                ui.add_space(8.0);

                let trust_lbl = self.tr("trust_device");
                ui.checkbox(&mut self.login_trust, trust_lbl);
                if ui.small_button(self.tr("details")).clicked() {
                    self.help_key = "help_trust";
                }
            }
            ui.add_space(14.0);
            if ui.add_sized([280.0, 34.0], egui::Button::new(self.tr("open_vault"))).clicked() {
                submit = true;
            }
            ui.add_space(6.0);
            if ui.button(self.tr("back")).clicked() {
                self.screen = Screen::Start;
                self.error.clear();
            }
            if submit {
                self.do_login();
            }
            if !self.error.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&self.error).color(pal().danger));
            }
        });
    }

    fn do_login(&mut self) {
        self.error.clear();
        if !self.vault_exists() {
            self.error = self.tr("vault_not_found");
            return;
        }
        let uses_ak = securevault::vault_uses_account_key(&self.vault_path);
        let trusted = Vault::is_trusted(&device_store(&self.vault_path));
        let need_ak = uses_ak && !trusted;
        let warn = securevault::integrity_changed(&self.vault_path, &device_store(&self.vault_path));
        let ds = device_store(&self.vault_path);

        let res = if need_ak {
            Vault::unlock(&self.vault_path, ds, self.password.as_bytes(), &self.account_key.clone(), self.login_trust)
        } else {
            Vault::open(&self.vault_path, ds, self.password.as_bytes())
        };
        match res {
            Ok(v) => {
                self.vault = Some(v);
                self.password.clear();
                self.account_key.clear();
                self.integrity_warn = warn;
                self.integrity_ack = false;
                if self.has_totp() {
                    self.awaiting_totp = true;
                    self.need_focus = true;
                } else {
                    self.awaiting_totp = false;
                }
                self.awaiting_bio = self.has_biometric();
                self.screen = Screen::Main;
                self.want_focus = Focus::AddContent;
                self.remember_vault();
                self.touch();

                if let Some(s) = self.vault.as_ref().map(|v| v.session_settings().clone()) {
                    self.sess_trust_days = s.trust_duration_days;
                    self.sess_lock_min = s.auto_lock_minutes.to_string();
                }
            }
            Err(e) => self.error = self.localize_err(&format!("{e:#}")),
        }
    }

    fn reg_pass_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(self.tr("language"));
            if ui.selectable_label(self.lang == Lang::En, "EN").clicked() {
                self.lang = Lang::En;
                self.config.language = "en".into();
                save_config(&self.config);
            }
            if ui.selectable_label(self.lang == Lang::Ru, "RU").clicked() {
                self.lang = Lang::Ru;
                self.config.language = "ru".into();
                save_config(&self.config);
            }
        });
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(self.tr("create_title")).size(22.0).strong().color(pal().accent));
            ui.add_space(8.0);
            ui.label(egui::RichText::new(self.tr("choose_password")).color(pal().muted));
            ui.add_space(14.0);
            let h_pass = t(self.lang, "password");
            let h_rep = t(self.lang, "repeat_password");
            let p1 = ui.add_sized([280.0, 32.0], egui::TextEdit::singleline(&mut self.reg_pass).password(true).hint_text(h_pass));
            if self.need_focus {
                p1.request_focus();
                self.need_focus = false;
            }
            let go1 = p1.lost_focus() && key_pressed(ui, egui::Key::Enter);
            ui.add_space(8.0);
            let p2 = ui.add_sized([280.0, 32.0], egui::TextEdit::singleline(&mut self.reg_pass2).password(true).hint_text(h_rep));
            let go2 = p2.lost_focus() && key_pressed(ui, egui::Key::Enter);
            ui.add_space(14.0);
            let next = ui.add_sized([280.0, 34.0], egui::Button::new(self.tr("next"))).clicked() || go1 || go2;
            ui.add_space(6.0);
            if ui.button(self.tr("back")).clicked() {
                self.screen = Screen::Start;
                self.error.clear();
            }
            if next {
                if self.reg_pass.is_empty() {
                    self.error = self.tr("pwd_empty");
                } else if self.reg_pass != self.reg_pass2 {
                    self.error = self.tr("pwd_mismatch");
                } else {
                    self.error.clear();
                    self.do_register();
                }
            }
            if !self.error.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&self.error).color(pal().danger));
            }
        });
    }

    fn do_register(&mut self) {
        let ds = device_store(&self.vault_path);
        match Vault::init(&self.vault_path, ds, self.reg_pass.as_bytes()) {
            Ok(v) => {
                self.vault = Some(v);
                self.reg_pass2.clear();
                self.ak_show.clear();
                self.reg_key_confirm.clear();
                self.reg_entropy.clear();
                self.reg_last_pos = None;
                self.reg_move_dist = 0.0;
                self.error.clear();
                self.info.clear();
                self.remember_vault();
                self.touch();
                self.sess_trust_days = 7;
                self.sess_lock_min = "15".into();

                self.screen = Screen::RegKey;
            }
            Err(e) => {
                self.error = format!("{e:#}");
                self.screen = Screen::Start;
            }
        }
    }

    fn reg_key_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(self.tr("reg_key_title")).size(22.0).strong().color(pal().accent));
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(self.tr("reg_key_intro")).color(pal().muted));
        ui.add_space(12.0);

        if self.ak_show.is_empty() {

            const ENTROPY_GOAL: f32 = 2500.0;
            if let Some(p) = ui.input(|i| i.pointer.latest_pos()) {
                if let Some(lp) = self.reg_last_pos {
                    let d = (p - lp).length();
                    if d > 0.0 {
                        self.reg_move_dist += d;
                        if self.reg_entropy.len() < 512 {
                            for b in (d.to_bits()).to_le_bytes() {
                                self.reg_entropy.push(b);
                            }
                            self.reg_entropy.push((p.x.to_bits() & 0xff) as u8);
                            self.reg_entropy.push((p.y.to_bits() & 0xff) as u8);
                        }
                    }
                }
                self.reg_last_pos = Some(p);
            }
            let progress = (self.reg_move_dist / ENTROPY_GOAL).min(1.0);
            let ready = self.reg_move_dist >= ENTROPY_GOAL && !self.reg_entropy.is_empty();
            ui.ctx().request_repaint();
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(self.tr("entropy_hint")).color(pal().muted));
                ui.add_space(8.0);
                ui.add_sized([300.0, 22.0], egui::ProgressBar::new(progress).show_percentage());
                ui.add_space(12.0);
                if ui.add_enabled(ready, egui::Button::new(self.tr("reg_key_create"))).clicked() {
                    let pw = self.reg_pass.clone();
                    let entropy = self.reg_entropy.clone();
                    let trust = self.reg_trust;
                    let res = self.vault.as_mut().map(|v| v.enable_account_key(pw.as_bytes(), &entropy, trust));
                    match res {
                        Some(Ok(k)) => {
                            self.ak_show = k;
                            self.reg_key_confirm.clear();
                            self.error.clear();
                        }
                        Some(Err(e)) => self.error = format!("{e:#}"),
                        None => {}
                    }
                }
                if !ready {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(self.tr("entropy_move")).size(12.0).color(pal().muted));
                }
            });
        } else {
            let key = self.ak_show.clone();
            let copy_lbl = self.tr("copy");
            egui::Frame::none()
                .fill(pal().card2)
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(&key).monospace().size(14.0));
                    ui.add_space(6.0);
                    if ui.button(copy_lbl).clicked() {
                        ui.output_mut(|o| o.copied_text = key.clone());
                        self.info = self.tr("copied");
                    }
                });
            ui.add_space(8.0);
            ui.label(egui::RichText::new(self.tr("reg_key_save_warn")).color(pal().danger));
            ui.add_space(6.0);

            let trust_lbl = self.tr("trust_device");
            ui.checkbox(&mut self.reg_trust, trust_lbl);
            ui.label(egui::RichText::new(self.tr("trust_hint")).size(11.0).color(pal().muted));
            ui.add_space(6.0);

            if ui.button(self.tr("download_sheet")).clicked() {
                let sheet = format!(
                    "H.R.E.N. vault — аварийный лист восстановления\n\nAccount Key:\n{}\n\nЭтот ключ нужен для входа с нового устройства и восстановления доступа.\nХраните его ОФЛАЙН (сейф/бумага). Никому не показывайте.\nБез ключа и пароля данные восстановить НЕЛЬЗЯ.\n",
                    key
                );
                if let Some(p) = rfd::FileDialog::new().set_file_name("hren-recovery.txt").save_file() {
                    if std::fs::write(&p, sheet).is_ok() {
                        self.info = self.tr("sheet_saved");
                    }
                }
            }
            ui.add_space(8.0);
            let paste_hint = self.tr("reg_key_paste");
            let r = ui.add_sized([ui.available_width(), 30.0], egui::TextEdit::singleline(&mut self.reg_key_confirm).hint_text(paste_hint));
            let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
            let matches = self.reg_key_confirm.trim() == key.trim() && !key.is_empty();
            ui.add_space(8.0);
            if ui.add_enabled(matches, egui::Button::new(self.tr("reg_key_continue"))).clicked() || (enter && matches) {
                self.reg_pass.clear();
                self.ak_show.clear();
                self.reg_key_confirm.clear();
                self.reg_entropy.clear();
                self.reg_last_pos = None;
                self.reg_move_dist = 0.0;
                self.info.clear();
                self.screen = Screen::Main;
                self.want_focus = Focus::AddContent;
            }
        }
        if !self.error.is_empty() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(&self.error).color(pal().danger));
        }
        if !self.info.is_empty() {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(&self.info).color(pal().green));
        }
    }

    fn add_entry(&mut self) {
        self.error.clear();
        match self.new_type {
            NewType::Note => {
                let content = self.new_content.trim().to_string();
                if content.is_empty() {
                    self.error = self.tr("fill_note");
                    return;
                }
                let desc = if self.new_desc.trim().is_empty() {
                    auto_describe(&content)
                } else {
                    self.new_desc.trim().to_string()
                };
                let hints = connectors(&content);
                self.store_record("note", &desc, &content, hints);
            }
            NewType::Secret => {
                let desc = self.new_desc.trim().to_string();
                let content = self.new_content.trim().to_string();
                if desc.is_empty() {
                    self.error = self.tr("secret_need_desc");
                    self.want_focus = Focus::AddDesc;
                    return;
                }
                if content.is_empty() {
                    self.error = self.tr("fill_secret");
                    self.want_focus = Focus::AddContent;
                    return;
                }
                if desc == content {
                    self.error = self.tr("desc_ne_content");
                    return;
                }
                self.store_record("secret", &desc, &content, Vec::new());
            }
        }
    }

    fn store_record(&mut self, rtype: &str, desc: &str, content: &str, hints: Vec<String>) {
        if let Some(v) = self.vault.as_mut() {
            let mut map = serde_json::Map::new();
            map.insert("content".into(), serde_json::Value::String(content.to_string()));
            match v.add_record(rtype, desc, serde_json::Value::Object(map), hints) {
                Ok(_) => {
                    self.new_desc.clear();
                    self.new_content.clear();
                    self.info = self.tr("record_added");
                }
                Err(e) => self.error = format!("{e:#}"),
            }
        }
    }

    fn open_record(&mut self, id: &str) {
        let res = self.vault.as_ref().map(|v| {
            let created_lbl = t(self.lang, "created_at");
            let meta = v.records().iter().find(|r| r.id == id).map(|r| {

                let dt = r.created.chars().take(16).collect::<String>().replace('T', " ");
                format!("{}: {}", created_lbl, dt)
            }).unwrap_or_default();
            v.get_record(id).map(|val| (content_text(&val), meta))
        });
        match res {
            Some(Ok((content, meta))) => {
                self.selected_orig = content.clone();
                self.selected_content = content;
                self.selected_meta = meta;
                self.selected_id = Some(id.to_string());
            }
            Some(Err(e)) => self.error = format!("{e:#}"),
            None => {}
        }
    }

    fn header(&self, ui: &mut egui::Ui) -> bool {
        let mut open_settings = false;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("H.R.E.N.™").size(18.0).strong().color(pal().accent));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(self.tr("settings")).clicked() {
                    open_settings = true;
                }
            });
        });
        ui.add_space(4.0);
        ui.separator();
        open_settings
    }

    fn add_form(&mut self, ui: &mut egui::Ui) {
        let mut do_add = false;
        let sel = if self.new_type == NewType::Note { self.tr("note") } else { self.tr("secret") };
        let lbl_note = self.tr("note");
        let lbl_secret = self.tr("secret");
        let lbl_type = self.tr("type");
        let lbl_add = self.tr("add");
        ui.horizontal(|ui| {
            ui.label(lbl_type);
            egui::ComboBox::from_id_salt("newtype")
                .selected_text(sel)
                .width(110.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.new_type, NewType::Note, lbl_note).clicked() {
                        self.want_focus = Focus::AddContent;
                    }
                    if ui.selectable_value(&mut self.new_type, NewType::Secret, lbl_secret).clicked() {
                        self.want_focus = Focus::AddDesc;
                    }
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_sized([150.0, 42.0], egui::Button::new(egui::RichText::new(lbl_add).size(18.0).strong()))
                    .clicked()
                {
                    do_add = true;
                }
            });
        });

        if self.new_type == NewType::Secret {
            let h_desc = self.tr("secret_desc_hint");
            let h_content = self.tr("secret_content_hint");
            let h_note = self.tr("secret_note");
            let d = ui.add_sized([ui.available_width(), 30.0], egui::TextEdit::singleline(&mut self.new_desc).hint_text(h_desc));
            self.focus(Focus::AddDesc, &d);
            if d.has_focus() && key_pressed(ui, egui::Key::ArrowDown) {
                self.want_focus = Focus::AddContent;
            }
            if d.lost_focus() && key_pressed(ui, egui::Key::Enter) {
                self.want_focus = Focus::AddContent;
            }
            let c = ui.add_sized([ui.available_width(), 30.0], egui::TextEdit::singleline(&mut self.new_content).password(true).hint_text(h_content));
            self.focus(Focus::AddContent, &c);
            if c.has_focus() && key_pressed(ui, egui::Key::ArrowUp) {
                self.want_focus = Focus::AddDesc;
            }
            if c.lost_focus() && key_pressed(ui, egui::Key::Enter) {
                do_add = true;
            }
            ui.label(egui::RichText::new(h_note).size(11.0).color(pal().muted));
        } else {
            let h_desc = self.tr("note_desc_hint");
            let h_content = self.tr("note_content_hint");
            let d = ui.add_sized([ui.available_width(), 30.0], egui::TextEdit::singleline(&mut self.new_desc).hint_text(h_desc));
            self.focus(Focus::AddDesc, &d);
            if d.has_focus() && key_pressed(ui, egui::Key::ArrowDown) {
                self.want_focus = Focus::AddContent;
            }
            if d.lost_focus() && key_pressed(ui, egui::Key::Enter) {
                self.want_focus = Focus::AddContent;
            }
            let c = ui.add_sized([ui.available_width(), 30.0], egui::TextEdit::singleline(&mut self.new_content).hint_text(h_content));
            self.focus(Focus::AddContent, &c);
            if c.has_focus() && key_pressed(ui, egui::Key::ArrowUp) {
                self.want_focus = Focus::AddDesc;
            }
            if c.lost_focus() && key_pressed(ui, egui::Key::Enter) {
                do_add = true;
            }
        }
        if do_add {
            self.add_entry();
        }
    }

    fn main_ui(&mut self, ui: &mut egui::Ui) {
        if self.header(ui) {
            self.show_settings = true;
            self.open_section = SettingsSection::None;
        }
        if self.awaiting_totp {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(self.tr("twofa_check")).size(18.0).strong());
                ui.add_space(6.0);
                ui.label(egui::RichText::new(self.tr("enter_code")).color(pal().muted));
                ui.add_space(10.0);
                let r = ui.add_sized([200.0, 32.0], egui::TextEdit::singleline(&mut self.totp_code).hint_text("000000"));
                if self.need_focus {
                    r.request_focus();
                    self.need_focus = false;
                }
                let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                ui.add_space(8.0);
                if ui.button(self.tr("confirm")).clicked() || enter {
                    let ok = self.totp_secret().is_some_and(|s| securevault::totp_verify(&s, &self.totp_code));
                    if ok {
                        self.awaiting_totp = false;
                        self.totp_code.clear();
                        self.error.clear();
                        self.want_focus = Focus::AddContent;
                    } else {
                        self.error = self.tr("wrong_2fa");
                    }
                }
                ui.add_space(4.0);
                if ui.button(self.tr("exit")).clicked() {
                    self.vault = None;
                    self.awaiting_totp = false;
                    self.totp_code.clear();
                    self.screen = Screen::Start;
                }
                if !self.error.is_empty() {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(&self.error).color(pal().danger));
                }
            });
            return;
        }
        if self.awaiting_bio {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(self.tr("biometrics")).size(18.0).strong());
                ui.add_space(6.0);
                ui.label(egui::RichText::new(self.tr("bio_confirm_login")).color(pal().muted));
                ui.add_space(10.0);
                if ui.button(self.tr("confirm")).clicked() {
                    let reason = self.tr("bio_reason_login");
                    self.start_bio(BioAction::Unlock, &reason);
                }
                if self.bio_rx.is_some() {
                    ui.label(egui::RichText::new(self.tr("bio_waiting")).color(pal().muted));
                }
                ui.add_space(4.0);
                if ui.button(self.tr("exit")).clicked() {
                    self.vault = None;
                    self.awaiting_bio = false;
                    self.screen = Screen::Start;
                }
                if !self.error.is_empty() {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(&self.error).color(pal().danger));
                }
            });
            return;
        }
        if !self.show_settings {
        if self.integrity_warn && !self.integrity_ack {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(90, 30, 30))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(self.tr("integrity_warn")).color(Color32::from_rgb(255, 180, 180)));
                    if ui.button(self.tr("got_it")).clicked() {
                        self.integrity_ack = true;
                    }
                });
        }
        ui.add_space(6.0);

        let search_hint = self.tr("search_hint");
        egui::Frame::none()
            .fill(pal().card2)
            .stroke(egui::Stroke::new(1.5, pal().accent))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.query).hint_text(search_hint).desired_width(f32::INFINITY).frame(false));
            });
        ui.add_space(8.0);

        self.add_form(ui);

        ui.add_space(8.0);

        let sort_lbl = if self.sort_oldest { self.tr("sort_oldest") } else { self.tr("sort_newest") };
        let filt_lbl = match self.filter_kind {
            1 => self.tr("filter_notes"),
            2 => self.tr("filter_secrets"),
            _ => self.tr("filter_all"),
        };
        let menu_lbl = format!("{}:  {}  ·  {}", self.tr("sort"), sort_lbl, filt_lbl);
        ui.menu_button(menu_lbl, |ui| {
            if ui.selectable_label(!self.sort_oldest, self.tr("sort_newest")).clicked() { self.sort_oldest = false; }
            if ui.selectable_label(self.sort_oldest, self.tr("sort_oldest")).clicked() { self.sort_oldest = true; }
            ui.separator();
            if ui.selectable_label(self.filter_kind == 0, self.tr("filter_all")).clicked() { self.filter_kind = 0; }
            if ui.selectable_label(self.filter_kind == 1, self.tr("filter_notes")).clicked() { self.filter_kind = 1; }
            if ui.selectable_label(self.filter_kind == 2, self.tr("filter_secrets")).clicked() { self.filter_kind = 2; }
        });
        ui.add_space(6.0);

        let q = self.query.trim().to_lowercase();
        let fk = self.filter_kind;
        let mut items: Vec<(String, String, String, Vec<String>, String)> = self
            .vault
            .as_ref()
            .map(|v| {
                v.records()
                    .iter()
                    .filter(|r| {
                        if r.rec_type == "totp_secret" || r.rec_type == "biometric" {
                            return false;
                        }
                        let is_secret = r.rec_type != "note";
                        if fk == 1 && is_secret { return false; }
                        if fk == 2 && !is_secret { return false; }
                        if !q.is_empty() {
                            let hit = r.title.to_lowercase().contains(&q)
                                || r.search_hints.iter().any(|h| h.to_lowercase().contains(&q));
                            if !hit { return false; }
                        }
                        true
                    })
                    .map(|r| (r.id.clone(), r.rec_type.clone(), r.title.clone(), r.search_hints.clone(), r.created.clone()))
                    .collect()
            })
            .unwrap_or_default();

        items.sort_by(|a, b| a.4.cmp(&b.4));
        if !self.sort_oldest {
            items.reverse();
        }

        let mut to_open: Option<(String, bool)> = None;
        let mut to_delete: Option<String> = None;
        let lbl_empty = self.tr("empty");
        let lbl_secret_sub = self.tr("list_secret_sub");
        let lbl_open = self.tr("open");
        let lbl_delete = self.tr("delete");

        let title_col = if self.config.theme == "light" { Color32::BLACK } else { pal().text };
        let list_h = (ui.available_height() - 70.0).max(180.0);

        egui::ScrollArea::vertical().max_height(list_h).auto_shrink([false, true]).show(ui, |ui| {
            if items.is_empty() {
                ui.label(egui::RichText::new(lbl_empty.as_str()).color(pal().muted));
            }
            for (id, rtype, title, _hints, created) in &items {
                let secret = rtype != "note";
                egui::Frame::none()
                    .fill(pal().card)
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {

                                ui.add(egui::Label::new(egui::RichText::new(title).size(14.0).strong().color(title_col)).wrap_mode(egui::TextWrapMode::Truncate));
                                let sub = if secret {
                                    lbl_secret_sub.clone()
                                } else {
                                    String::new()
                                };
                                if !sub.is_empty() {
                                    ui.add(egui::Label::new(egui::RichText::new(sub).size(12.0).color(pal().muted)).wrap_mode(egui::TextWrapMode::Truncate));
                                }
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(lbl_delete.as_str()).clicked() {
                                    to_delete = Some(id.clone());
                                }
                                if ui.button(lbl_open.as_str()).clicked() {
                                    to_open = Some((id.clone(), secret));
                                }
                            });
                        });

                        let date = created.chars().take(10).collect::<String>();
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(date).size(10.0).color(pal().muted));
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        });

        if let Some((id, secret)) = to_open {
            if secret {
                self.pending_open = Some(id);
                self.open_password.clear();
                self.want_focus = Focus::OpenPass;
                self.popup_armed = false;
                self.error.clear();
            } else {
                self.open_record(&id);
            }
        }
        if let Some(id) = to_delete {
            self.pending_delete = Some(id);
            self.del_password.clear();
            self.want_focus = Focus::DelPass;
            self.popup_armed = false;
            self.error.clear();
        }

        if let Some(id) = self.pending_open.clone() {
            let mut submit = false;
            let mut cancel = false;
            let inner = egui::Window::new("open_secret_popup")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .default_width(360.0)
                .show(ui.ctx(), |ui| {
                    ui.label(egui::RichText::new(self.tr("open_secret_confirm")).strong());
                    ui.add_space(6.0);
                    let hp = t(self.lang, "password");
                    let r = ui.add(egui::TextEdit::singleline(&mut self.open_password).password(true).hint_text(hp));
                    self.focus(Focus::OpenPass, &r);
                    let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button(t(self.lang, "open")).clicked() || enter { submit = true; }
                        if ui.button(t(self.lang, "cancel")).clicked() { cancel = true; }
                    });
                    if !self.error.is_empty() {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&self.error).size(12.0).color(pal().danger));
                    }
                });

            let outside = if self.popup_armed {
                inner.as_ref().map(|i| i.response.clicked_elsewhere()).unwrap_or(false)
            } else {
                self.popup_armed = true;
                false
            };
            if submit {
                let pw = self.open_password.clone();
                let ok = self.vault.as_ref().map(|v| v.verify_password(pw.as_bytes())).unwrap_or(false);
                if ok {
                    self.open_record(&id);
                    self.pending_open = None;
                    self.open_password.clear();
                    self.error.clear();
                } else {
                    self.error = self.tr("wrong_password");
                    self.open_password.clear();
                }
            } else if cancel || outside {
                self.pending_open = None;
                self.open_password.clear();
                self.error.clear();
            }
        }

        if let Some(id) = self.pending_delete.clone() {
            let mut confirm = false;
            let mut cancel = false;
            let inner = egui::Window::new("delete_secret_popup")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .default_width(360.0)
                .show(ui.ctx(), |ui| {
                    ui.label(egui::RichText::new(self.tr("delete_record_confirm")).color(pal().danger).strong());
                    ui.add_space(6.0);
                    let hp = t(self.lang, "password");
                    let r = ui.add(egui::TextEdit::singleline(&mut self.del_password).password(true).hint_text(hp));
                    self.focus(Focus::DelPass, &r);
                    let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button(t(self.lang, "delete")).clicked() || enter { confirm = true; }
                        if ui.button(t(self.lang, "cancel")).clicked() { cancel = true; }
                    });
                    if !self.error.is_empty() {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&self.error).size(12.0).color(pal().danger));
                    }
                });
            let outside = if self.popup_armed {
                inner.as_ref().map(|i| i.response.clicked_elsewhere()).unwrap_or(false)
            } else {
                self.popup_armed = true;
                false
            };
            if confirm {
                let pw = self.del_password.clone();
                let ok = self.vault.as_ref().map(|v| v.verify_password(pw.as_bytes())).unwrap_or(false);
                if ok {
                    if let Some(v) = self.vault.as_mut() {
                        let _ = v.delete_record(&id);
                    }
                    self.selected_content.clear();
                    self.info = self.tr("deleted_record");
                    self.error.clear();
                    self.pending_delete = None;
                    self.del_password.clear();
                } else {
                    self.error = self.tr("wrong_password");
                    self.del_password.clear();
                }
            } else if cancel || outside {
                self.pending_delete = None;
                self.del_password.clear();
                self.error.clear();
            }
        }

        if !self.selected_content.is_empty() {
            let mut close = false;
            let mut do_save = false;
            egui::Window::new(t(self.lang, "content"))
                .collapsible(false)
                .resizable(true)
                .default_width(460.0)
                .default_height(380.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    if !self.selected_meta.is_empty() {
                        ui.label(egui::RichText::new(&self.selected_meta).size(12.0).color(pal().muted));
                        ui.add_space(6.0);
                    }
                    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.selected_content).desired_width(f32::INFINITY));
                    });
                    ui.add_space(8.0);
                    let changed = self.selected_content != self.selected_orig;
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t(self.lang, "close")).clicked() {
                            close = true;
                        }
                        if ui.add_enabled(changed, egui::Button::new(t(self.lang, "save"))).clicked() {
                            do_save = true;
                        }
                    });
                });
            if do_save {
                if let Some(id) = self.selected_id.clone() {
                    let content = self.selected_content.clone();
                    match self.vault.as_mut().map(|v| v.update_record(&id, &content)) {
                        Some(Ok(())) => self.info = self.tr("saved"),
                        Some(Err(e)) => self.error = format!("{e:#}"),
                        None => {}
                    }
                }
                close = true;
            }
            if close {
                self.selected_content.clear();
                self.selected_meta.clear();
                self.selected_orig.clear();
                self.selected_id = None;
            }
        }
        }

        if self.show_settings {
            let lang0 = self.lang;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(t(lang0, "settings")).size(22.0).strong().color(pal().accent));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t(lang0, "back")).clicked() {
                        self.show_settings = false;
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let title_lang = t(lang0, "language");
                if self.section(ui, SettingsSection::Language, &title_lang) {
                    egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(self.lang == Lang::En, "EN").clicked() {
                                self.lang = Lang::En;
                                self.config.language = "en".into();
                                save_config(&self.config);
                            }
                            if ui.selectable_label(self.lang == Lang::Ru, "RU").clicked() {
                                self.lang = Lang::Ru;
                                self.config.language = "ru".into();
                                save_config(&self.config);
                            }
                        });
                    });
                }
                let title_disp = t(lang0, "appearance");
                if self.section(ui, SettingsSection::Display, &title_disp) {
                    egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {

                        ui.label(t(lang0, "theme"));
                        let is_light = self.config.theme == "light";
                        ui.horizontal(|ui| {
                            if ui.selectable_label(!is_light, t(lang0, "theme_dark")).clicked() {
                                self.config.theme = "dark".into();
                                set_palette(false);
                                self.themed = false;
                                save_config(&self.config);
                            }
                            if ui.selectable_label(is_light, t(lang0, "theme_light")).clicked() {
                                self.config.theme = "light".into();
                                set_palette(true);
                                self.themed = false;
                                save_config(&self.config);
                            }
                        });
                        ui.add_space(8.0);

                        ui.label(format!("{}: {}%", t(lang0, "scale"), (self.config.ui_scale * 100.0).round() as i32));
                        let mut s = self.config.ui_scale;
                        if ui.add(egui::Slider::new(&mut s, 0.8..=1.6).show_value(false)).changed() {
                            self.config.ui_scale = (s * 20.0).round() / 20.0;
                            save_config(&self.config);
                        }
                    });
                }
                let title_pw = t(lang0, "change_credentials");
                if self.section(ui, SettingsSection::ChangePw, &title_pw) {
            let lang = self.lang;
            ui.add_space(6.0);
            egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {

                ui.horizontal(|ui| {
                    if ui.button(t(lang, "change_password")).clicked() {
                        self.cred_open = if self.cred_open == 1 { 0 } else { 1 };
                        self.error.clear();
                    }
                    if ui.button(t(lang, "change_key")).clicked() {
                        self.cred_open = if self.cred_open == 2 { 0 } else { 2 };
                        self.ak_rotate_pw.clear();
                        self.ak_show.clear();
                        self.error.clear();
                    }
                });

                if self.cred_open == 1 {
                    ui.add_space(8.0);
                    if ui.small_button(t(lang, "details")).clicked() {
                        self.help_key = "help_password";
                    }
                    let h_new = t(lang, "new_password");
                    let h_rep = t(lang, "repeat_password");
                    let h_ak = t(lang, "ak_for_change");
                    let _r1 = ui.add(egui::TextEdit::singleline(&mut self.np1).password(true).hint_text(h_new));
                    let r2 = ui.add(egui::TextEdit::singleline(&mut self.np2).password(true).hint_text(h_rep));
                    ui.add(egui::TextEdit::singleline(&mut self.np_ak).hint_text(h_ak));
                    let save = ui.button(t(lang, "save_password")).clicked() || (r2.lost_focus() && key_pressed(ui, egui::Key::Enter));
                    if save {
                        if self.np1.is_empty() || self.np1 != self.np2 {
                            self.error = self.tr("pwd_pair_bad");
                        } else {
                            let ak = self.np_ak.clone();
                            if let Some(v) = self.vault.as_mut() {
                                match v.change_password(self.np1.as_bytes(), Some(ak.as_str())) {
                                    Ok(()) => {
                                        self.info = self.tr("password_changed");
                                        self.np1.clear();
                                        self.np2.clear();
                                        self.np_ak.clear();
                                    }
                                    Err(e) => self.error = format!("{e:#}"),
                                }
                            }
                        }
                    }
                }

                if self.cred_open == 2 {
                    ui.add_space(8.0);
                    if ui.small_button(t(lang, "details")).clicked() {
                        self.help_key = "help_changekey";
                    }
                    if self.ak_enabled() {

                        ui.label(egui::RichText::new(t(lang, "ak_rotate_warn")).color(pal().danger));
                        let r = ui.add(egui::TextEdit::singleline(&mut self.ak_rotate_pw).password(true).hint_text(t(lang, "password")));
                        let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                        if ui.button(t(lang, "ak_reissue_btn")).clicked() || enter {
                            let pw = self.ak_rotate_pw.clone();
                            let res = self.vault.as_mut().map(|v| v.rotate_account_key(pw.as_bytes()));
                            match res {
                                Some(Ok(k)) => {
                                    self.ak_show = k;
                                    self.ak_rotate_pw.clear();
                                    self.error.clear();
                                    self.info = self.tr("ak_reissued");
                                }
                                Some(Err(e)) => {
                                    self.error = format!("{e:#}");
                                    self.ak_rotate_pw.clear();
                                }
                                None => {}
                            }
                        }
                    } else {

                        ui.label(egui::RichText::new(t(lang, "ak_enable_hint")).size(12.0).color(pal().muted));
                        let r = ui.add(egui::TextEdit::singleline(&mut self.ak_rotate_pw).password(true).hint_text(t(lang, "password")));
                        let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                        if ui.button(t(lang, "ak_enable")).clicked() || enter {
                            let pw = self.ak_rotate_pw.clone();
                            let res = self.vault.as_mut().map(|v| v.enable_account_key(pw.as_bytes(), &[], true));
                            match res {
                                Some(Ok(k)) => {
                                    self.ak_show = k;
                                    self.ak_rotate_pw.clear();
                                    self.error.clear();
                                    self.info = self.tr("ak_enabled_ok");
                                }
                                Some(Err(e)) => {
                                    self.error = format!("{e:#}");
                                    self.ak_rotate_pw.clear();
                                }
                                None => {}
                            }
                        }
                    }
                    if !self.ak_show.is_empty() {
                        ui.label(egui::RichText::new(t(lang, "ak_save_once")).color(pal().danger).size(12.0));
                        ui.label(egui::RichText::new(&self.ak_show).monospace().size(14.0));
                        ui.horizontal(|ui| {
                            if ui.button(t(lang, "copy")).clicked() {
                                ui.output_mut(|o| o.copied_text = self.ak_show.clone());
                                self.info = self.tr("copied");
                            }
                            if ui.button(t(lang, "done")).clicked() {
                                self.ak_show.clear();
                            }
                        });
                    }
                }
            });
        }

                let title_totp = t(lang0, "totp_title");
                if self.section(ui, SettingsSection::Totp, &title_totp) {
            ui.add_space(6.0);
            egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                if ui.small_button(t(lang0, "details")).clicked() {
                    self.help_key = "help_totp";
                }
                if self.has_totp() {
                    ui.label(egui::RichText::new(t(lang0, "on")).color(pal().green));
                    ui.horizontal(|ui| {
                        if ui.button(t(lang0, "totp_rebind")).clicked() {
                            self.rebind_confirm = true;
                            self.rebind_password.clear();
                            self.error.clear();
                        }
                        if ui.button(t(lang0, "totp_disable")).clicked() {
                            self.disable_2fa_confirm = true;
                            self.dis_pw.clear();
                            self.error.clear();
                        }
                    });
                    if self.disable_2fa_confirm {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(t(lang0, "disable_2fa_confirm")).color(pal().danger));
                        let r = ui.add(egui::TextEdit::singleline(&mut self.dis_pw).password(true).hint_text(t(lang0, "password")));
                        let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                        ui.horizontal(|ui| {
                            if ui.button(t(lang0, "confirm_disable")).clicked() || enter {
                                let pw = self.dis_pw.clone();
                                let ok = self.vault.as_ref().is_some_and(|v| v.verify_password(pw.as_bytes()));
                                if ok {
                                    let id = self.vault.as_ref().and_then(|v| v.records().iter().find(|r| r.rec_type == "totp_secret").map(|r| r.id.clone()));
                                    if let Some(id) = id {
                                        if let Some(v) = self.vault.as_mut() {
                                            let _ = v.delete_record(&id);
                                        }
                                    }
                                    self.totp_setup.clear();
                                    self.disable_2fa_confirm = false;
                                    self.dis_pw.clear();
                                    self.error.clear();
                                    self.info = self.tr("twofa_disabled");
                                } else {
                                    self.error = self.tr("wrong_password");
                                    self.dis_pw.clear();
                                }
                            }
                            if ui.button(t(lang0, "cancel")).clicked() {
                                self.disable_2fa_confirm = false;
                                self.dis_pw.clear();
                            }
                        });
                    }
                    if self.rebind_confirm {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(t(lang0, "rebind_warn")).color(pal().danger));
                        let r = ui.add(egui::TextEdit::singleline(&mut self.rebind_password).password(true).hint_text(t(lang0, "password")));
                        let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                        ui.horizontal(|ui| {
                            if ui.button(t(lang0, "confirm")).clicked() || enter {
                                let pw = self.rebind_password.clone();
                                let ok = self.vault.as_ref().is_some_and(|v| v.verify_password(pw.as_bytes()));
                                if ok {
                                    let id = self
                                        .vault
                                        .as_ref()
                                        .and_then(|v| v.records().iter().find(|r| r.rec_type == "totp_secret").map(|r| r.id.clone()));
                                    if let Some(id) = id {
                                        if let Some(v) = self.vault.as_mut() {
                                            let _ = v.delete_record(&id);
                                        }
                                    }
                                    let b32 = securevault::totp_new_secret_base32();
                                    if let Some(v) = self.vault.as_mut() {
                                        let mut map = serde_json::Map::new();
                                        map.insert("secret".into(), serde_json::Value::String(b32.clone()));
                                        let _ = v.add_record("totp_secret", "2FA TOTP", serde_json::Value::Object(map), vec![]);
                                    }
                                    self.totp_setup = b32;
                                    self.rebind_confirm = false;
                                    self.rebind_password.clear();
                                    self.error.clear();
                                    self.info = self.tr("secret_reissued");
                                } else {
                                    self.error = self.tr("wrong_password");
                                    self.rebind_password.clear();
                                }
                            }
                            if ui.button(t(lang0, "cancel")).clicked() {
                                self.rebind_confirm = false;
                                self.rebind_password.clear();
                            }
                        });
                    }
                } else if ui.button(t(lang0, "totp_enable")).clicked() {
                    let b32 = securevault::totp_new_secret_base32();
                    if let Some(v) = self.vault.as_mut() {
                        let mut map = serde_json::Map::new();
                        map.insert("secret".into(), serde_json::Value::String(b32.clone()));
                        let _ = v.add_record("totp_secret", "2FA TOTP", serde_json::Value::Object(map), vec![]);
                    }
                    self.totp_setup = b32;
                }

                if !self.totp_setup.is_empty() {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(t(lang0, "totp_scan")).size(12.0).color(pal().muted));
                    let url = securevault::totp_url(&self.totp_setup, "user");
                    draw_qr(ui, &url);
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(lang0, "totp_manual")).size(12.0).color(pal().muted));
                    ui.label(egui::RichText::new(&self.totp_setup).monospace());
                    if ui.button(t(lang0, "done")).clicked() {
                        self.totp_setup.clear();
                    }
                }
            });
        }

                let title_bio = t(lang0, "bio_title");
                if self.section(ui, SettingsSection::Bio, &title_bio) {
            ui.add_space(6.0);
            egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                if ui.small_button(t(lang0, "details")).clicked() {
                    self.help_key = "help_bio";
                }
                ui.label(egui::RichText::new(t(lang0, "bio_desc2")).size(12.0).color(pal().muted));
                ui.add_space(4.0);
                if self.has_biometric() {
                    ui.label(egui::RichText::new(t(lang0, "on")).color(pal().green));
                    if ui.button(t(lang0, "bio_disable")).clicked() {
                        self.disable_bio_confirm = true;
                        self.bio_pw.clear();
                        self.error.clear();
                    }
                    if self.disable_bio_confirm {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(t(lang0, "bio_disable_confirm")).color(pal().danger));
                        let r = ui.add(egui::TextEdit::singleline(&mut self.bio_pw).password(true).hint_text(t(lang0, "password")));
                        let enter = r.lost_focus() && key_pressed(ui, egui::Key::Enter);
                        ui.horizontal(|ui| {
                            if ui.button(t(lang0, "confirm")).clicked() || enter {
                                let pw = self.bio_pw.clone();
                                let ok = self.vault.as_ref().is_some_and(|v| v.verify_password(pw.as_bytes()));
                                if ok {
                                    let id = self.vault.as_ref().and_then(|v| v.records().iter().find(|r| r.rec_type == "biometric").map(|r| r.id.clone()));
                                    if let Some(id) = id {
                                        if let Some(v) = self.vault.as_mut() {
                                            let _ = v.delete_record(&id);
                                        }
                                    }
                                    self.disable_bio_confirm = false;
                                    self.bio_pw.clear();
                                    self.error.clear();
                                    self.info = self.tr("bio_disabled");
                                } else {
                                    self.error = self.tr("wrong_password");
                                    self.bio_pw.clear();
                                }
                            }
                            if ui.button(t(lang0, "cancel")).clicked() {
                                self.disable_bio_confirm = false;
                                self.bio_pw.clear();
                            }
                        });
                    }
                } else if self.bio_available {
                    if ui.button(t(lang0, "bio_enable")).clicked() {
                        let reason = t(lang0, "bio_reason_enable");
                        self.start_bio(BioAction::Enable, &reason);
                    }
                    if self.bio_rx.is_some() {
                        ui.label(egui::RichText::new(t(lang0, "bio_waiting")).color(pal().muted));
                    }
                } else {
                    ui.label(egui::RichText::new(t(lang0, "bio_unavail")).size(12.0).color(pal().muted));
                }
            });
        }

                let title_sess = t(lang0, "session");
                if self.section(ui, SettingsSection::Session, &title_sess) {
            ui.add_space(6.0);
            let mut do_save_sess = false;
            let ak_on_sess = self.ak_enabled();
            let totp_on_sess = self.has_totp();
            let h_ak = "Account Key".to_string();
            let h_2fa = t(lang0, "totp_code");
            let warn = t(lang0, "session_warn");
            egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                if ui.small_button(t(lang0, "details")).clicked() {
                    self.help_key = "help_session";
                }
                ui.horizontal(|ui| {
                    ui.label(t(lang0, "trust_days"));
                    for d in [1u32, 3, 7] {
                        if ui.selectable_label(self.sess_trust_days == d, format!("{d}")).clicked() {
                            self.sess_trust_days = d;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(t(lang0, "auto_lock"));
                    ui.add(egui::TextEdit::singleline(&mut self.sess_lock_min).desired_width(60.0));
                });
                if ak_on_sess || totp_on_sess {
                    ui.label(egui::RichText::new(warn).size(11.0).color(pal().muted));
                }
                if ak_on_sess {
                    ui.add(egui::TextEdit::singleline(&mut self.sess_ak).hint_text(h_ak));
                }
                if totp_on_sess {
                    ui.add(egui::TextEdit::singleline(&mut self.sess_totp).hint_text(h_2fa));
                }
                if ui.button(t(lang0, "save")).clicked() {
                    do_save_sess = true;
                }
            });
            if do_save_sess {
                let ak_ok = if ak_on_sess {
                    self.vault.as_ref().is_some_and(|v| v.verify_account_key(&self.sess_ak))
                } else {
                    true
                };
                let totp_ok = if totp_on_sess {
                    self.totp_secret().is_some_and(|s| securevault::totp_verify(&s, &self.sess_totp))
                } else {
                    true
                };
                if !ak_ok {
                    self.error = self.tr("wrong_ak");
                } else if !totp_ok {
                    self.error = self.tr("wrong_2fa");
                } else {
                    let mins: u32 = self.sess_lock_min.trim().parse().unwrap_or(15);
                    self.sess_lock_min = mins.to_string();
                    let days = self.sess_trust_days.max(1);
                    if let Some(v) = self.vault.as_mut() {
                        match v.set_session_settings(days, mins) {
                            Ok(()) => {
                                self.info = self.tr("settings_saved");
                                self.sess_ak.clear();
                                self.sess_totp.clear();
                            }
                            Err(e) => self.error = format!("{e:#}"),
                        }
                    }
                }
            }
        }

                let title_acc = t(lang0, "account");
                if self.section(ui, SettingsSection::Account, &title_acc) {
            ui.add_space(6.0);
            egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                if ui.small_button(t(lang0, "details")).clicked() {
                    self.help_key = "help_account";
                }
                ui.horizontal(|ui| {
                    if ui.button(t(lang0, "sign_out")).clicked() {
                        self.confirm_signout = true;
                    }
                    if ui.add(egui::Button::new(egui::RichText::new(t(lang0, "delete_account")).color(pal().danger))).clicked() {
                        self.confirm_delete = true;
                        self.del_pw.clear();
                        self.del_ak.clear();
                        self.del_totp.clear();
                        self.error.clear();
                    }
                });
            });
        }

        if self.confirm_delete {
            let needs_ak = self.ak_enabled();
            ui.add_space(6.0);
            egui::Frame::none().fill(Color32::from_rgb(90, 30, 30)).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                ui.label(egui::RichText::new(t(lang0, "wipe_title")).strong().color(Color32::from_rgb(255, 180, 180)));
                ui.label(egui::RichText::new(t(lang0, "wipe_warn")).color(Color32::from_rgb(255, 200, 200)));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(t(lang0, "confirm_all_factors")).size(12.0).color(pal().muted));
                ui.add(egui::TextEdit::singleline(&mut self.del_pw).password(true).hint_text(t(lang0, "password")));
                if needs_ak {
                    ui.add(egui::TextEdit::singleline(&mut self.del_ak).hint_text("Account Key"));
                }
                if self.has_totp() {
                    ui.add(egui::TextEdit::singleline(&mut self.del_totp).hint_text(t(lang0, "totp_code")));
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new(t(lang0, "wipe_btn")).color(pal().danger))).clicked() {
                        let pw_ok = self.vault.as_ref().is_some_and(|v| v.verify_password(self.del_pw.as_bytes()));
                        let ak_ok = if needs_ak {
                            self.vault.as_ref().is_some_and(|v| v.verify_account_key(&self.del_ak))
                        } else { true };
                        let totp_ok = if self.has_totp() {
                            self.totp_secret().is_some_and(|s| securevault::totp_verify(&s, &self.del_totp))
                        } else {
                            true
                        };
                        if pw_ok && ak_ok && totp_ok {
                            let _ = securevault::wipe_account(&self.vault_path, &device_store(&self.vault_path));
                            self.vault = None;
                            self.confirm_delete = false;
                            self.show_settings = false;
                            self.selected_content.clear();
                            self.del_pw.clear();
                            self.del_ak.clear();
                            self.del_totp.clear();
                            self.error.clear();
                            self.info = self.tr("account_wiped");
                            self.screen = Screen::Start;
                        } else if !pw_ok {
                            self.error = self.tr("wrong_password");
                        } else if !ak_ok {
                            self.error = self.tr("wrong_ak");
                        } else {
                            self.error = self.tr("wrong_2fa");
                        }
                    }
                    if ui.button(t(lang0, "cancel")).clicked() {
                        self.confirm_delete = false;
                        self.del_pw.clear();
                        self.del_ak.clear();
                        self.del_totp.clear();
                    }
                });
            });
        }

        if self.confirm_signout {
            let ak_on_signout = self.ak_enabled();
            ui.add_space(4.0);
            egui::Frame::none().fill(pal().card2).rounding(egui::Rounding::same(10.0)).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                if ak_on_signout {
                    ui.label(egui::RichText::new(t(lang0, "signout_warn_ak")).color(pal().danger));
                } else {
                    ui.label(egui::RichText::new(t(lang0, "signout_warn_base")).color(pal().muted));
                }
                ui.horizontal(|ui| {
                    if ui.button(t(lang0, "signout_yes")).clicked() {
                        let _ = securevault::untrust_device(&device_store(&self.vault_path));
                        self.vault = None;
                        self.selected_content.clear();
                        self.show_settings = false;
                        self.pending_delete = None;
                        self.pending_open = None;
                        self.confirm_signout = false;
                        self.error.clear();
                        self.info = if ak_on_signout {
                            t(lang0, "signed_out_ak")
                        } else {
                            t(lang0, "signed_out_base")
                        };
                        self.screen = Screen::Start;
                    }
                    if ui.button(t(lang0, "cancel")).clicked() {
                        self.confirm_signout = false;
                    }
                });
            });
        }
                ui.add_space(10.0);
                if !self.info.is_empty() {
                    ui.label(egui::RichText::new(&self.info).color(pal().green));
                }
                if !self.error.is_empty() {
                    ui.label(egui::RichText::new(&self.error).color(pal().danger));
                }
            });
        }

        if !self.info.is_empty() || !self.error.is_empty() {
            ui.add_space(8.0);
            ui.separator();
            if !self.info.is_empty() {
                ui.label(egui::RichText::new(&self.info).size(12.0).color(pal().green));
            }
            if !self.error.is_empty() {
                ui.label(egui::RichText::new(&self.error).size(12.0).color(pal().danger));
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);

        if (self.applied_scale - self.config.ui_scale).abs() > f32::EPSILON {
            ctx.set_zoom_factor(self.config.ui_scale);
            self.applied_scale = self.config.ui_scale;
        }

        if self.screen == Screen::Splash {
            ctx.request_repaint();
            if self.splash_start.is_none_or(|s| s.elapsed().as_secs_f32() >= 1.2) {
                self.screen = Screen::Start;
            }
        }

        if self.vault.is_some() {
            let minimized = ctx.input(|i| i.viewport().minimized).unwrap_or(false);
            if minimized {
                self.vault = None;
                self.selected_content.clear();
                self.show_settings = false;
                self.awaiting_totp = false;
                self.awaiting_bio = false;
                self.open_section = SettingsSection::None;
                self.last_active = None;
                self.info = self.tr("locked_min");
                self.screen = Screen::Start;
            }
        }

        if self.screen == Screen::Main && self.vault.is_some() {
            let active = ctx.input(|i| !i.events.is_empty());
            if active {
                self.last_active = Some(std::time::Instant::now());
            }
            let mins = self.vault.as_ref().map(|v| v.session_settings().auto_lock_minutes).unwrap_or(0);
            if mins > 0 {
                if let Some(t0) = self.last_active {
                    if t0.elapsed().as_secs() >= (mins as u64) * 60 {
                        self.vault = None;
                        self.selected_content.clear();
                        self.show_settings = false;
                        self.awaiting_totp = false;
                        self.awaiting_bio = false;
                        self.last_active = None;
                        self.info = self.tr("auto_locked");
                        self.screen = Screen::Start;
                    }
                }
                ctx.request_repaint_after(std::time::Duration::from_secs(5));
            }
        }

        if self.bio_rx.is_some() {
            ctx.request_repaint();
            let res = match self.bio_rx.as_ref().unwrap().try_recv() {
                Ok(r) => Some(r),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(false),
            };
            if let Some(r) = res {
                self.bio_rx = None;
                match self.bio_action {
                    BioAction::Enable => {
                        if r {
                            if let Some(v) = self.vault.as_mut() {
                                let _ = v.add_record("biometric", "Биометрия", serde_json::Value::Object(serde_json::Map::new()), vec![]);
                            }
                            self.info = self.tr("bio_enabled_msg");
                            self.error.clear();
                        } else {
                            self.error = self.tr("bio_not_confirmed");
                        }
                    }
                    BioAction::Unlock => {
                        if r {
                            self.awaiting_bio = false;
                            self.error.clear();
                        } else {
                            self.error = self.tr("bio_not_confirmed");
                        }
                    }
                    BioAction::None => {}
                }
                self.bio_action = BioAction::None;
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Splash => self.splash_ui(ui),
            Screen::Start => self.start_ui(ui),
            Screen::Login => self.login_ui(ui),
            Screen::RegPass => self.reg_pass_ui(ui),
            Screen::RegKey => self.reg_key_ui(ui),
            Screen::Main => self.main_ui(ui),
        });

        if let Some(key) = self.current_help() {
            let lang = self.lang;
            let title = t(lang, "details_title");
            let body = t(lang, key);
            let armed = self.help_armed == self.help_key;
            egui::Window::new("help_popup")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .default_width(460.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(title.as_str()).size(16.0).strong().color(pal().accent));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(body).size(15.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(t(lang, "tap_to_close")).size(11.0).color(pal().muted));
                });
            if !armed {
                self.help_armed = self.help_key;
            } else if ctx.input(|i| i.pointer.any_click()) {
                self.help_key = "";
                self.help_armed = "";
            }
        }
    }
}

fn load_app_icon() -> Option<egui::IconData> {
    const ICON_PNG: &[u8] = include_bytes!("../../assets/hren_icon.png");
    eframe::icon_data::from_png_bytes(ICON_PNG).ok()
}

fn main() -> eframe::Result<()> {
    let mut vb = egui::ViewportBuilder::default()
        .with_title("H.R.E.N. vault\u{2122} (beta)")
        .with_inner_size([520.0, 700.0])
        .with_min_inner_size([400.0, 520.0]);
    if let Some(icon) = load_app_icon() {
        vb = vb.with_icon(std::sync::Arc::new(icon));
    }
    let opts = eframe::NativeOptions { viewport: vb, ..Default::default() };
    eframe::run_native("H.R.E.N. vault", opts, Box::new(|_cc| Ok(Box::new(App::new()))))
}
