$ErrorActionPreference = "Stop"

if (-not (Test-Path ".\Cargo.toml")) {
    throw "Execute este script na raiz do projeto, onde esta o Cargo.toml."
}

function New-RandomSecret {
    $bytes = New-Object byte[] 32
    $rng = [Security.Cryptography.RandomNumberGenerator]::Create()

    try {
        $rng.GetBytes($bytes)
    }
    finally {
        $rng.Dispose()
    }

    return ([BitConverter]::ToString($bytes)).Replace("-", "").ToLowerInvariant()
}

function Ensure-LocalEnv {
    if (Test-Path ".\.env") {
        return
    }

    Write-Host "Arquivo .env nao encontrado. Criando configuracao local segura..." -ForegroundColor Yellow

    $jwtSecret = New-RandomSecret
    $adminSecret = New-RandomSecret

    @"
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
JWT_SECRET=$jwtSecret
ADMIN_SECRET=$adminSecret
COOKIE_SECURE=false
"@ | Set-Content ".\.env" -Encoding ascii
}

function Import-DotEnv {
    if (-not (Test-Path ".\.env")) {
        throw "Arquivo .env nao encontrado."
    }

    Get-Content ".\.env" | ForEach-Object {
        $line = $_.Trim()

        if (
            $line.Length -gt 0 -and
            -not $line.StartsWith("#") -and
            $line.Contains("=")
        ) {
            $name, $value = $line -split "=", 2
            $name = $name.Trim()
            $value = $value.Trim()

            if (
                ($value.StartsWith('"') -and $value.EndsWith('"')) -or
                ($value.StartsWith("'") -and $value.EndsWith("'"))
            ) {
                $value = $value.Substring(1, $value.Length - 2)
            }

            Set-Item -Path "Env:$name" -Value $value
        }
    }

    if (-not $env:DATABASE_URL) {
        throw "DATABASE_URL nao foi carregada do .env."
    }
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
        throw "Docker Desktop nao foi encontrado no Windows."
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

    throw "O Docker nao ficou disponivel dentro de 3 minutos."
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

    throw "O PostgreSQL nao ficou disponivel dentro de 2 minutos."
}

Ensure-LocalEnv
Import-DotEnv

Write-Host "Ambiente local carregado." -ForegroundColor Green

Start-DockerDesktopIfNeeded

Write-Host "Iniciando banco de dados..." -ForegroundColor Cyan
docker compose up -d

if ($LASTEXITCODE -ne 0) {
    throw "Nao foi possivel iniciar o banco de dados."
}

Wait-Postgres

if (-not (Get-Command sqlx -ErrorAction SilentlyContinue)) {
    throw "O sqlx-cli nao esta instalado. Execute: cargo install sqlx-cli --no-default-features --features postgres"
}

Write-Host "Aplicando migrations..." -ForegroundColor Cyan
sqlx migrate run

if ($LASTEXITCODE -ne 0) {
    throw "Falha ao aplicar migrations."
}

Write-Host "Iniciando o Investment OS..." -ForegroundColor Green
Write-Host "Abra http://localhost:3000" -ForegroundColor Cyan
Write-Host "Use Ctrl + C para encerrar o servidor." -ForegroundColor Yellow

cargo run
