@echo off
echo === Build ediliyor... ===
cargo build --release
if %errorlevel% neq 0 pause && exit /b %errorlevel%

echo === Dosyalar kopyalaniyor... ===
copy /Y "target\release\rustkeyboard.exe" "C:\rustkeyboard\rustkeyboard.exe"

echo === Tamamlandi! ===
echo C:\rustkeyboard\rustkeyboard.exe
pause
