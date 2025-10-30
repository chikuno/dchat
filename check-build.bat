@echo off
REM Simple batch script to wait for dchat containers to be ready

setlocal enabledelayedexpansion

set START_TIME=%time%
set TIMEOUT=1800
set CHECK_INTERVAL=10

echo.
echo ====================================
echo   DCHAT BUILD STATUS CHECKER
echo ====================================
echo.
echo Started: %DATE% %START_TIME%
echo.

:LOOP
for /f "tokens=*" %%A in ('docker-compose ps 2^>nul ^| findstr /c:"Up" ^| find /c /v ""') do set COUNT=%%A

if "%COUNT%"=="7" (
    echo.
    echo [SUCCESS] All 7 containers are running!
    echo.
    docker-compose ps
    echo.
    echo Access your services:
    echo   - Grafana: http://localhost:3000 (admin/admin^)
    echo   - Prometheus: http://localhost:9090
    echo   - Jaeger: http://localhost:16686
    echo.
    goto END
)

echo Checking... (Found %COUNT%/7 containers running^)
timeout /t %CHECK_INTERVAL% /nobreak
goto LOOP

:END
echo.
echo Build completed successfully!
pause
