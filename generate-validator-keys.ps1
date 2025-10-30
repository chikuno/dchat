#!/usr/bin/env pwsh
# Generate validator keys for testnet

$KeyDir = "validator_keys"
New-Item -ItemType Directory -Force -Path $KeyDir | Out-Null

Write-Host "Generating validator keys..." -ForegroundColor Green

# Generate unencrypted keys by using echo to provide empty password
for ($i = 1; $i -le 4; $i++) {
    Write-Host "Generating validator$i key..."
    # Use Write-Output with empty password to generate unencrypted keys
    Write-Output "" | docker run --rm -i -v "$(Get-Location)\${KeyDir}:/keys" dchat:latest keygen --output "/keys/validator$i.key" 2>&1 | Select-Object -Last 3
}

Write-Host "✓ All validator keys generated" -ForegroundColor Green
Get-ChildItem $KeyDir | Format-Table -Property Name, Length
