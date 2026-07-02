# Contributing / Как контрибьютить

Thanks for your interest! / Спасибо за интерес!

## Ground rules / Основные правила

- **EN:** By submitting a contribution you agree it is licensed under the project's license (GNU AGPL‑3.0) and you sign off each commit under the Developer Certificate of Origin (DCO): commit with `git commit -s` (adds a `Signed-off-by` line).
- **RU:** Отправляя правку, вы соглашаетесь, что она лицензируется под лицензией проекта (GNU AGPL‑3.0), и подписываете каждый коммит по DCO: коммитьте с `git commit -s` (добавляет строку `Signed-off-by`).

## Before a PR / Перед PR

- `cargo build --release` and `cargo test` pass / проходят.
- `cargo clippy` without new warnings / без новых предупреждений.
- Keep UI strings bilingual (EN + RU) / держите строки интерфейса двуязычными.
- Do not add telemetry or send user data anywhere / не добавляйте телеметрию и не отправляйте данные пользователя.

## Security

For vulnerabilities see `SECURITY.md` — report privately, not via public issues.
Для уязвимостей см. `SECURITY.md` — сообщайте приватно, не через публичные issue.
