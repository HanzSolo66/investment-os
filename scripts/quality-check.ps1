$ErrorActionPreference = "Stop"

if (-not (Test-Path ".\Cargo.toml")) {
    throw "Execute este script na raiz do projeto."
}

docker info *> $null

if ($LASTEXITCODE -ne 0) {
    throw "Abra o Docker Desktop antes de executar a verificação."
}

docker compose up -d

for ($attempt = 1; $attempt -le 60; $attempt++) {
    docker compose exec -T db pg_isready -U postgres -d postgres *> $null

    if ($LASTEXITCODE -eq 0) {
        break
    }

    Start-Sleep -Seconds 2
}

if ($LASTEXITCODE -ne 0) {
    throw "O PostgreSQL não ficou disponível."
}

sqlx migrate run
cargo fmt --check
cargo check
cargo test

Write-Host ""
Write-Host "Qualidade aprovada." -ForegroundColor Green
