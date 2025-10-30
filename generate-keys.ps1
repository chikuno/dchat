#!/usr/bin/env pwsh
# Generate validator keys in the correct format for dchat

param(
    [string]$OutputDir = "validator_keys",
    [int]$Count = 4
)

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Write-Host "Generating $Count validator keys..." -ForegroundColor Green

# Generate raw bytes and format them for each validator
for ($i = 1; $i -le $Count; $i++) {
    $OutputFile = Join-Path $OutputDir "validator$i.key"
    
    Write-Host "Generating validator$i..."
    
    # Generate 32 random bytes for Ed25519 key
    $randomBytes = New-Object byte[] 32
    $rng = New-Object System.Security.Cryptography.RNGCryptoServiceProvider
    $rng.GetBytes($randomBytes)
    
    # Format as debug array string
    $byteArray = ($randomBytes | ForEach-Object { $_.ToString() }) -join ", "
    
    # Create JSON with private_key field in debug format
    $keyJson = @{
        private_key = "[$byteArray]"
        generated_at = (Get-Date -Format "o")
        validator_id = "validator$i"
    } | ConvertTo-Json
    
    # Write to file
    Set-Content -Path $OutputFile -Value $keyJson
    
    Write-Host "  ✓ Generated $OutputFile"
}

Write-Host "`n✓ All validator keys generated" -ForegroundColor Green
Write-Host "`nValidator keys:" -ForegroundColor Cyan
Get-ChildItem $OutputDir | Format-Table -Property Name, Length, LastWriteTime
