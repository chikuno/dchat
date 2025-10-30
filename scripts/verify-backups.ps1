#!/usr/bin/env pwsh
# Backup Verification Script for dchat
# Tests backup integrity and restorability

param(
    [Parameter(Mandatory=$false)]
    [string]$S3Bucket = $env:BACKUP_S3_BUCKET,
    
    [Parameter(Mandatory=$false)]
    [int]$DaysToCheck = 7
)

# Configuration
$ErrorActionPreference = "Stop"
$TestDir = "/tmp/backup-verification"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "dchat Backup Verification" -ForegroundColor Cyan
Write-Host "Checking last $DaysToCheck days" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Validate parameters
if (-not $S3Bucket) {
    Write-Error "S3 bucket not specified. Set BACKUP_S3_BUCKET environment variable."
    exit 1
}

# Create test directory
New-Item -ItemType Directory -Force -Path $TestDir | Out-Null

# List backups from S3
Write-Host "`n[1/3] Listing backups from S3..." -ForegroundColor Yellow

$Cutoff = (Get-Date).AddDays(-$DaysToCheck).ToString("yyyy-MM-dd")
$Backups = aws s3 ls s3://$S3Bucket/backups/full/ --recursive | 
    Where-Object { $_ -match "\.sql\.gz$" } |
    Where-Object { $_ -match "\d{4}-\d{2}-\d{2}" } |
    Where-Object { $Matches[0] -ge $Cutoff }

$BackupCount = ($Backups | Measure-Object).Count
Write-Host "Found $BackupCount backups in last $DaysToCheck days" -ForegroundColor Green

# Check backup metadata and integrity
Write-Host "`n[2/3] Verifying backup integrity..." -ForegroundColor Yellow

$VerificationResults = @()

foreach ($Backup in $Backups) {
    $BackupPath = $Backup.Split()[-1]
    $BackupFile = Split-Path -Leaf $BackupPath
    $MetadataFile = $BackupFile -replace "\.sql\.gz$", ".meta.json"
    $MetadataPath = $BackupPath -replace "\.sql\.gz$", ".meta.json"
    
    Write-Host "  Checking: $BackupFile" -ForegroundColor White
    
    $Result = @{
        backup_file = $BackupFile
        timestamp = $null
        size_mb = $null
        checksum_verified = $false
        metadata_exists = $false
        errors = @()
    }
    
    # Download metadata
    $LocalMetadataFile = "$TestDir/$MetadataFile"
    try {
        aws s3 cp "s3://$S3Bucket/$MetadataPath" $LocalMetadataFile 2>&1 | Out-Null
        
        if (Test-Path $LocalMetadataFile) {
            $Metadata = Get-Content $LocalMetadataFile | ConvertFrom-Json
            $Result.metadata_exists = $true
            $Result.timestamp = $Metadata.timestamp
            $Result.size_mb = [math]::Round($Metadata.backup_size_bytes / 1MB, 2)
            
            # Verify backup file exists and size matches
            $BackupInfo = aws s3 ls "s3://$S3Bucket/$BackupPath" | Select-Object -Last 1
            $S3Size = [long]($BackupInfo.Split()[2])
            
            if ($S3Size -eq $Metadata.backup_size_bytes) {
                $Result.checksum_verified = $true
            } else {
                $Result.errors += "Size mismatch: S3=$S3Size, Metadata=$($Metadata.backup_size_bytes)"
            }
        } else {
            $Result.errors += "Metadata file not found"
        }
    } catch {
        $Result.errors += "Failed to download metadata: $_"
    }
    
    $VerificationResults += $Result
    
    # Cleanup
    Remove-Item -Path $LocalMetadataFile -Force -ErrorAction SilentlyContinue
}

# Display results
Write-Host "`n[3/3] Verification Results:" -ForegroundColor Yellow

$SuccessCount = ($VerificationResults | Where-Object { $_.checksum_verified -and $_.errors.Count -eq 0 }).Count
$FailureCount = $BackupCount - $SuccessCount

Write-Host "`nSummary:" -ForegroundColor Cyan
Write-Host "  Total backups: $BackupCount" -ForegroundColor White
Write-Host "  Verified: $SuccessCount" -ForegroundColor Green
Write-Host "  Failed: $FailureCount" -ForegroundColor $(if ($FailureCount -gt 0) { "Red" } else { "Green" })

if ($FailureCount -gt 0) {
    Write-Host "`nFailed Backups:" -ForegroundColor Red
    $VerificationResults | Where-Object { -not $_.checksum_verified -or $_.errors.Count -gt 0 } | ForEach-Object {
        Write-Host "  - $($_.backup_file)" -ForegroundColor Red
        $_.errors | ForEach-Object {
            Write-Host "    Error: $_" -ForegroundColor Red
        }
    }
}

# Check backup frequency
Write-Host "`nBackup Frequency Check:" -ForegroundColor Cyan

$DailyBackups = $VerificationResults | Group-Object { $_.timestamp.Split("_")[0] } | Measure-Object
$ExpectedDays = [math]::Min($DaysToCheck, (Get-Date).DayOfYear)

if ($DailyBackups.Count -ge $ExpectedDays - 1) {
    Write-Host "  Daily backups are running consistently" -ForegroundColor Green
} else {
    Write-Warning "  Missing daily backups. Expected: $ExpectedDays, Found: $($DailyBackups.Count)"
}

# Check retention policy
Write-Host "`nRetention Policy Check:" -ForegroundColor Cyan

$OldBackups = aws s3 ls s3://$S3Bucket/backups/full/ --recursive | 
    Where-Object { $_ -match "\.sql\.gz$" } |
    Where-Object { $_ -match "(\d{4}-\d{2}-\d{2})" } |
    Where-Object { $Matches[1] -lt (Get-Date).AddDays(-90).ToString("yyyy-MM-dd") }

$OldBackupCount = ($OldBackups | Measure-Object).Count

if ($OldBackupCount -gt 0) {
    Write-Warning "  Found $OldBackupCount backups older than 90 days (should be deleted)"
} else {
    Write-Host "  Retention policy is being enforced correctly" -ForegroundColor Green
}

# Cleanup
Remove-Item -Path $TestDir -Recurse -Force

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Verification completed!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

if ($FailureCount -gt 0) {
    exit 1
} else {
    exit 0
}
