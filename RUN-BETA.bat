@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo ============================================
echo   H.R.E.N. vault BETA - sborka i zapusk
echo ============================================
echo.
echo [1/3] Sborka (cargo build --release)...
cargo build --release
if errorlevel 1 (
  echo.
  echo !!! BUILD FAILED - skopiruy ves tekst oshibki vyshe i prishli Claude.
  echo.
  pause
  exit /b 1
)
echo.
echo [2/3] Sozdayu yarlyk "H.R.E.N. vault BETA" na rabochem stole...
powershell -NoProfile -Command "$d=[Environment]::GetFolderPath('Desktop'); $s=(New-Object -ComObject WScript.Shell).CreateShortcut($d+'\H.R.E.N. vault BETA.lnk'); $s.TargetPath='%~dp0target\release\sv_gui.exe'; $s.IconLocation='%~dp0assets\hren_icon.ico'; $s.WorkingDirectory='%~dp0target\release'; $s.Save()"
echo.
echo [3/3] Zapusk...
start "" "%~dp0target\release\sv_gui.exe"
echo Gotovo. Na rabochem stole - yarlyk "H.R.E.N. vault BETA".
timeout /t 3 >nul
