$ErrorActionPreference = "Stop"

if (-not (Test-Path ".\Cargo.toml")) {
    throw "Execute este script na raiz do projeto."
}

function Start-DockerDesktopIfNeeded {
    docker info *> $null

    if ($LASTEXITCODE -eq 0) {
        return
    }

    $dockerApp = Get-StartApps |
        Where-Object { $_.Name -like "*Docker Desktop*" } |
        Select-Object -First 1

    if (-not $dockerApp) {
        throw "Docker Desktop não foi encontrado no Windows."
    }

    Start-Process explorer.exe "shell:AppsFolder\$($dockerApp.AppID)"

    Write-Host "Aguardando o Docker Desktop iniciar..." -ForegroundColor Cyan

    for ($attempt = 1; $attempt -le 90; $attempt++) {
        Start-Sleep -Seconds 2
        docker info *> $null

        if ($LASTEXITCODE -eq 0) {
            return
        }
    }

    throw "O Docker não ficou disponível dentro de 3 minutos."
}

function Wait-Postgres {
    Write-Host "Aguardando o PostgreSQL..." -ForegroundColor Cyan

    for ($attempt = 1; $attempt -le 60; $attempt++) {
        docker compose exec -T db pg_isready -U postgres -d postgres *> $null

        if ($LASTEXITCODE -eq 0) {
            return
        }

        Start-Sleep -Seconds 2
    }

    throw "O PostgreSQL não ficou disponível dentro de 2 minutos."
}

Start-DockerDesktopIfNeeded

Write-Host "Iniciando banco de dados..." -ForegroundColor Cyan
docker compose up -d

if ($LASTEXITCODE -ne 0) {
    throw "Não foi possível iniciar o banco de dados."
}

Wait-Postgres

if (-not (Get-Command sqlx -ErrorAction SilentlyContinue)) {
    throw "O sqlx-cli não está instalado. Execute: cargo install sqlx-cli --no-default-features --features postgres"
}

Write-Host "Aplicando migrations..." -ForegroundColor Cyan
sqlx migrate run

Write-Host "Iniciando o Investment OS..." -ForegroundColor Green
Write-Host "Use Ctrl + C para encerrar o servidor." -ForegroundColor Yellow

cargo run
