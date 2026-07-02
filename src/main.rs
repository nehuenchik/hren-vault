
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{self, Write};
use securevault::{DeviceStore, Vault};

fn main() {
    if let Err(e) = run() {
        eprintln!("Ошибка: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return Ok(());
    }
    let cmd = args[0].clone();
    let opts = Opts::parse(&args[1..]);
    let vault = opts.get_one("vault").unwrap_or_else(default_vault_path);

    let device_store = match opts.get_one("device") {
        Some(path) => DeviceStore::file(path),
        None => DeviceStore::keychain(vault.clone(), format!("{vault}.devicekey")),
    };
    let trust_desc = device_store.describe();

    match cmd.as_str() {
        "note" | "n" => {
            let positional = positional_args(&args[1..]);
            let text = if !positional.is_empty() {
                positional.join(" ")
            } else {
                ask_line("Что зашифровать и сохранить? ")?
            };
            if text.trim().is_empty() {
                return Err(anyhow!("пустой текст — нечего сохранять"));
            }
            let pw = get_password()?;
            let mut v = Vault::open_or_init(&vault, device_store, pw.as_bytes())?;
            let title: String = text.chars().take(40).collect();
            let mut map = serde_json::Map::new();
            map.insert("content".into(), serde_json::Value::String(text.clone()));
            let id = v.add_record("note", &title, serde_json::Value::Object(map), vec![])?;
            println!("Сохранено \u{2713}  ({id})");
        }
        "read" | "r" => {
            let v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            if v.records().is_empty() {
                println!("(пусто)");
            }
            for r in v.records() {
                let val = v.get_record(&r.id)?;
                match val.get("content").and_then(|c| c.as_str()) {
                    Some(c) => println!("• {c}"),
                    None => println!("• [{}] {} → {}", r.rec_type, r.title, val),
                }
            }
        }
        "init" => {
            let pw = get_password()?;
            Vault::init(&vault, device_store, pw.as_bytes())?;
            println!("Хранилище создано: {vault}");
            println!("Доступ — по паролю. Account Key (доверенные устройства) можно включить: `enablekey`.");
        }
        "unlock" => {
            let pw = get_password()?;
            let ak = ask_line("Введите Account Key: ")?;
            Vault::unlock(&vault, device_store, pw.as_bytes(), &ak, true)?;
            println!("Вход выполнен, устройство теперь доверенное \u{2713}");
            println!("Account Key сохранён локально: {trust_desc}. Дальше достаточно пароля.");
        }
        "passwd" | "password" => {
            let mut v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            let ak = if v.account_key_enabled() {
                Some(ask_line("Account Key (обязателен для смены пароля): ")?)
            } else {
                None
            };
            let np = ask_new_password()?;
            v.change_password(np.as_bytes(), ak.as_deref())?;
            println!("Пароль изменён \u{2713}");
        }
        "enablekey" => {
            let pw = get_password()?;
            let mut v = Vault::open(&vault, device_store, pw.as_bytes())?;
            let new_ak = v.enable_account_key(pw.as_bytes(), &[], true)?;
            println!("Account Key включён \u{2713} Это устройство теперь доверенное.");
            print_account_key(&new_ak, &vault, &trust_desc);
        }
        "disablekey" => {
            let pw = get_password()?;
            let mut v = Vault::open(&vault, device_store, pw.as_bytes())?;
            let ak = ask_line("Введите текущий Account Key: ")?;
            v.disable_account_key(pw.as_bytes(), &ak)?;
            println!("Account Key отключён \u{2713} Доступ теперь только по паролю.");
        }
        "newkey" | "rotate" => {
            let pw = get_password()?;
            let mut v = Vault::open(&vault, device_store, pw.as_bytes())?;
            let new_ak = v.rotate_account_key(pw.as_bytes())?;
            println!("Account Key перевыпущен \u{2713} Старый больше не действует.");
            print_account_key(&new_ak, &vault, &trust_desc);
        }
        "untrust" => {
            securevault::untrust_device(&device_store)?;
            println!("Готово: это устройство больше не доверенное (локальный Account Key удалён из {trust_desc}).");
            println!("Данные в хранилище не тронуты. Чтобы снова войти — `unlock` (пароль + Account Key).");
        }
        "status" => {
            if Vault::is_trusted(&device_store) {
                println!("Устройство: ДОВЕРЕННОЕ — нужен только пароль.");
                println!("Account Key хранится в: {trust_desc}");
            } else {
                println!("Устройство: НЕ доверенное — для входа нужны пароль и Account Key (`unlock`).");
            }
            println!("Хранилище: {vault}");
        }
        "add" => {
            let mut v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            let rec_type = opts.req("type")?;
            let title = opts.req("title")?;
            let mut map = match opts.get_one("data") {
                Some(d) => match serde_json::from_str::<serde_json::Value>(&d) {
                    Ok(serde_json::Value::Object(m)) => m,
                    Ok(_) => return Err(anyhow!("--data должен быть JSON-объектом")),
                    Err(e) => return Err(anyhow!("--data не JSON: {e}")),
                },
                None => serde_json::Map::new(),
            };
            for f in opts.get_many("field") {
                match f.split_once('=') {
                    Some((k, val)) => {
                        map.insert(k.trim().to_string(), serde_json::Value::String(val.to_string()));
                    }
                    None => return Err(anyhow!("--field должен быть в виде ключ=значение, получено: {f}")),
                }
            }
            let id = v.add_record(&rec_type, &title, serde_json::Value::Object(map), opts.get_many("hint"))?;
            println!("Добавлена запись: {id}");
        }
        "list" => {
            let v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            if v.records().is_empty() {
                println!("(пусто)");
            }
            for r in v.records() {
                println!("{}  [{}]  {}  hints={:?}", r.id, r.rec_type, r.title, r.search_hints);
            }
        }
        "get" => {
            let v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            let id = opts.req("id")?;
            println!("{}", serde_json::to_string_pretty(&v.get_record(&id)?)?);
        }
        "search" => {
            let v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            let q = opts.req("query")?;
            let found = v.search(&q);
            if found.is_empty() {
                println!("ничего не найдено по запросу: {q}");
            }
            for r in found {
                println!("{}  [{}]  {}", r.id, r.rec_type, r.title);
            }
        }
        "delete" | "del" => {
            let mut v = Vault::open(&vault, device_store, get_password()?.as_bytes())?;
            let id = if let Some(id) = opts.get_one("id") {
                id
            } else {
                let q = positional_args(&args[1..]).join(" ");
                if q.trim().is_empty() {
                    return Err(anyhow!("укажи что удалить: delete <текст> или --id chunk_..."));
                }
                let ql = q.to_lowercase();
                let matches: Vec<(String, String, String)> = v
                    .records()
                    .iter()
                    .filter(|r| {
                        r.title.to_lowercase().contains(&ql)
                            || r.search_hints.iter().any(|h| h.to_lowercase().contains(&ql))
                    })
                    .map(|r| (r.id.clone(), r.rec_type.clone(), r.title.clone()))
                    .collect();
                match matches.len() {
                    0 => return Err(anyhow!("ничего не найдено по: {q}")),
                    1 => matches[0].0.clone(),
                    _ => {
                        eprintln!("Найдено несколько — уточни через --id <chunk_...>:");
                        for (mid, t, title) in &matches {
                            eprintln!("  {mid}  [{t}]  {title}");
                        }
                        return Ok(());
                    }
                }
            };
            let title = v.records().iter().find(|r| r.id == id).map(|r| r.title.clone()).unwrap_or_default();
            let ans = ask_line(&format!("Удалить «{title}» ({id})? Напиши yes для подтверждения: "))?;
            if ans.trim() != "yes" {
                println!("Отменено.");
                return Ok(());
            }
            v.delete_record(&id)?;
            println!("Удалено \u{2713}");
        }
        other => {
            eprintln!("неизвестная команда: {other}\n");
            print_help();
        }
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "SecureVault CLI  (база: доступ по паролю; Account Key — опционально)\n\n\
         Базовые команды (нужен только пароль):\n  \
         note [текст]   записать фразу\n  \
         read           показать все записи\n  \
         delete <текст> удалить запись\n  \
         passwd         сменить пароль\n  \
         status         доверенное ли устройство\n\n\
         Account Key (опциональное усиление + доверенные устройства):\n  \
         enablekey      включить Account Key (это устройство станет доверенным)\n  \
         newkey         перевыпустить Account Key\n  \
         disablekey     отключить Account Key (нужны пароль + текущий ключ)\n  \
         untrust        снять доверие с этого устройства\n  \
         unlock         войти на чужом устройстве (пароль + Account Key)\n\n\
         Ещё: init / add --type <t> --title <s> [--field k=v ...] / list / get --id <..> / search --query <..>\n\
         Флаги: --vault <путь> --device <файл> (файловый режим доверия вместо OS Keychain)"
    );
}

fn print_account_key(ak: &str, vault: &str, trust_desc: &str) {
    println!("\n========= СОХРАНИ ACCOUNT KEY (ОДИН РАЗ) =========");
    println!("Account Key:");
    println!("    {ak}");
    println!("\nОн нужен ВМЕСТЕ с паролем для входа на НОВОМ устройстве.");
    println!("Храни его оффлайн (бумага), отдельно от пароля и не в этом же хранилище.");
    println!("\nНа этом устройстве копия Account Key сохранена в: {trust_desc}");
    println!("Хранилище: {vault}");
    println!("\nДоступ = пароль + Account Key. Забыл пароль — данные не вернуть.");
    println!("=================================================\n");
}

fn positional_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            if i + 1 < args.len() && !args[i + 1].starts_with("--") { i += 2; } else { i += 1; }
        } else {
            out.push(args[i].clone());
            i += 1;
        }
    }
    out
}

fn ask_line(prompt: &str) -> Result<String> {
    eprint!("{prompt}");
    io::stderr().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim_end_matches(['\n', '\r']).to_string())
}

fn get_password() -> Result<String> {
    if let Ok(p) = std::env::var("SECUREVAULT_PASSWORD") {
        return Ok(p);
    }
    eprint!("Пароль (ввод не отображается): ");
    io::stderr().flush().ok();
    Ok(rpassword::read_password()?)
}

fn ask_new_password() -> Result<String> {
    if let Ok(p) = std::env::var("SECUREVAULT_NEW_PASSWORD") {
        return Ok(p);
    }
    loop {
        eprint!("Новый пароль (ввод не отображается): ");
        io::stderr().flush().ok();
        let p1 = rpassword::read_password()?;
        eprint!("Повтори новый пароль: ");
        io::stderr().flush().ok();
        let p2 = rpassword::read_password()?;
        if p1 != p2 {
            eprintln!("Пароли не совпали, попробуй снова.");
            continue;
        }
        if p1.trim().is_empty() {
            eprintln!("Пустой пароль не годится.");
            continue;
        }
        return Ok(p1);
    }
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

struct Opts(HashMap<String, Vec<String>>);

impl Opts {
    fn parse(args: &[String]) -> Self {
        let mut m: HashMap<String, Vec<String>> = HashMap::new();
        let mut i = 0;
        while i < args.len() {
            if let Some(name) = args[i].strip_prefix("--") {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    m.entry(name.to_string()).or_default().push(args[i + 1].clone());
                    i += 2;
                } else {
                    m.entry(name.to_string()).or_default().push("true".into());
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        Opts(m)
    }
    fn get_one(&self, k: &str) -> Option<String> {
        self.0.get(k).and_then(|v| v.first().cloned())
    }
    fn get_many(&self, k: &str) -> Vec<String> {
        self.0.get(k).cloned().unwrap_or_default()
    }
    fn req(&self, k: &str) -> Result<String> {
        self.get_one(k).ok_or_else(|| anyhow!("обязательный флаг --{k} не задан"))
    }
}
