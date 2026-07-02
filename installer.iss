; Inno Setup script for H.R.E.N. vault (beta)
; Builds setup.exe with a wizard: welcome -> license -> install -> shortcut.
; Before compiling run: cargo build --release  (so target\release\sv_gui.exe exists)
; Then open this file in Inno Setup Compiler and press Compile (or Build -> Compile).

#define AppName "H.R.E.N. vault"
#define AppVersion "0.1.0-beta"
#define AppExe "sv_gui.exe"

[Setup]
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=H.R.E.N.
; Default install location: a folder on the user's Desktop.
DefaultDirName={userdesktop}\HREN vault
DisableProgramGroupPage=yes
; No administrator rights required (installs into a user folder).
PrivilegesRequired=lowest
; License page shown during install (user must accept).
LicenseFile=EULA.txt
OutputBaseFilename=HREN-vault-setup
SetupIconFile=assets\hren_icon.ico
UninstallDisplayIcon={app}\{#AppExe}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Files]
Source: "target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\hren_icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion

[Icons]
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\assets\hren_icon.ico"
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\assets\hren_icon.ico"

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch H.R.E.N. vault"; Flags: nowait postinstall skipifsilent
