; Inno Setup script для H.R.E.N. vault (beta)
; Собирает setup.exe с окном-мастером: приветствие -> соглашение -> установка -> ярлык.
; Перед компиляцией сделай: cargo build --release  (чтобы был target\release\sv_gui.exe)
; Затем открой этот файл в Inno Setup Compiler и нажми Compile (или Build -> Compile).

#define AppName "H.R.E.N. vault"
#define AppVersion "0.1.0-beta"
#define AppExe "sv_gui.exe"

[Setup]
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=H.R.E.N.
; По умолчанию ставим на рабочий стол, в подпапку "HREN vault".
DefaultDirName={userdesktop}\HREN vault
DisableProgramGroupPage=yes
; Не требуем прав администратора (ставим в пользовательскую папку).
PrivilegesRequired=lowest
; Окно с текстом соглашения (пользователь должен принять).
LicenseFile=EULA.txt
; Имя итогового файла установщика.
OutputBaseFilename=HREN-vault-setup
SetupIconFile=assets\hren_icon.ico
UninstallDisplayIcon={app}\{#AppExe}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\hren_icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion

[Icons]
; Ярлык на рабочем столе.
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\assets\hren_icon.ico"
; Ярлык в меню Пуск.
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\assets\hren_icon.ico"

[Run]
; Предложить запустить после установки.
Filename: "{app}\{#AppExe}"; Description: "Запустить H.R.E.N. vault"; Flags: nowait postinstall skipifsilent
