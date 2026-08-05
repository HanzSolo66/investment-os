$ErrorActionPreference = "Stop"

if (-not (Test-Path ".\Cargo.toml")) {
    throw "Execute este script na raiz do projeto, onde está o Cargo.toml."
}

function New-SecureSecret {
    $bytes = New-Object byte[] 48
    $generator = [System.Security.Cryptography.RandomNumberGenerator]::Create()

    try {
        $generator.GetBytes($bytes)
    }
    finally {
        $generator.Dispose()
    }

    return [Convert]::ToBase64String($bytes)
}

$databaseUrl = "postgres://postgres:postgres@localhost:5432/postgres"

if (Test-Path ".\.env") {
    $existingDatabaseUrl = Get-Content ".\.env" |
        Where-Object { $_ -match "^DATABASE_URL=" } |
        Select-Object -First 1

    if ($existingDatabaseUrl) {
        $databaseUrl = $existingDatabaseUrl.Substring(
            "DATABASE_URL=".Length
        )
    }
}

$jwtSecret = New-SecureSecret
$adminSecret = New-SecureSecret

$content = @"
DATABASE_URL=$databaseUrl
JWT_SECRET=$jwtSecret
ADMIN_SECRET=$adminSecret
COOKIE_SECURE=false
"@

$utf8WithoutBom = New-Object System.Text.UTF8Encoding($false)

[System.IO.File]::WriteAllText(
    (Join-Path (Get-Location) ".env"),
    $content,
    $utf8WithoutBom
)

Write-Host ""
Write-Host "Arquivo .env criado com segredos aleatórios." -ForegroundColor Green
Write-Host "COOKIE_SECURE ficou false para funcionar em localhost." -ForegroundColor Cyan
Write-Host "Em produção HTTPS, altere COOKIE_SECURE para true." -ForegroundColor Yellow
Write-Host ""
