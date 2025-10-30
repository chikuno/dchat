#!/usr/bin/env pwsh
# Automated Database Backup Script for dchat
# Performs full backups daily and incremental backups hourly
# Uploads to S3 with encryption

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("full", "incremental")]
    [string]$BackupType = "full",
    
    [Parameter(Mandatory=$false)]
    [string]$S3Bucket = $env:BACKUP_S3_BUCKET,
    
    [Parameter(Mandatory=$false)]
    [string]$DatabaseHost = $env:DATABASE_HOST,
    
    [Parameter(Mandatory=$false)]
    [string]$DatabaseName = $env:DATABASE_NAME,
    
    [Parameter(Mandatory=$false)]
    [string]$DatabaseUser = $env:DATABASE_USER,
    
    [Parameter(Mandatory=$false)]
    [string]$Namespace = "default"
)

# Configuration
$ErrorActionPreference = "Stop"
$Timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$BackupDir = "/tmp/backups"
$BackupFile = "$BackupDir/dchat-$BackupType-$Timestamp.sql.gz"
$MetadataFile = "$BackupDir/dchat-$BackupType-$Timestamp.meta.json"
$S3Prefix = "backups/$BackupType"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "dchat Database Backup" -ForegroundColor Cyan
Write-Host "Type: $BackupType" -ForegroundColor Cyan
Write-Host "Timestamp: $Timestamp" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Validate parameters
if (-not $S3Bucket) {
    Write-Error "S3 bucket not specified. Set BACKUP_S3_BUCKET environment variable."
    exit 1
}

if (-not $DatabaseHost) {
    Write-Error "Database host not specified. Set DATABASE_HOST environment variable."
    exit 1
}

# Create backup directory
New-Item -ItemType Directory -Force -Path $BackupDir | Out-Null

# Get pod name
Write-Host "`n[1/6] Finding dchat pod..." -ForegroundColor Yellow
$PodName = kubectl get pods -n $Namespace -l app.kubernetes.io/name=dchat -o jsonpath='{.items[0].metadata.name}'

if (-not $PodName) {
    Write-Error "No dchat pod found in namespace $Namespace"
    exit 1
}
Write-Host "Found pod: $PodName" -ForegroundColor Green

# Perform backup
Write-Host "`n[2/6] Creating database backup..." -ForegroundColor Yellow

if ($BackupType -eq "full") {
    # Full backup using pg_dump
    $DumpCommand = "pg_dump -h $DatabaseHost -U $DatabaseUser -d $DatabaseName --format=custom --compress=9"
    kubectl exec -n $Namespace $PodName -- bash -c "PGPASSWORD=`$DATABASE_PASSWORD $DumpCommand" | Out-File -FilePath $BackupFile -Encoding UTF8
} else {
    # Incremental backup using WAL archiving
    $WalCommand = "pg_receivewal -h $DatabaseHost -U $DatabaseUser -D /tmp/wal_archive --slot=dchat_backup --create-slot"
    kubectl exec -n $Namespace $PodName -- bash -c "PGPASSWORD=`$DATABASE_PASSWORD $WalCommand"
    kubectl exec -n $Namespace $PodName -- tar -czf - /tmp/wal_archive | Out-File -FilePath $BackupFile -Encoding UTF8
}

# Verify backup file exists and has content
if (-not (Test-Path $BackupFile)) {
    Write-Error "Backup file was not created: $BackupFile"
    exit 1
}

$BackupSize = (Get-Item $BackupFile).Length
if ($BackupSize -eq 0) {
    Write-Error "Backup file is empty"
    exit 1
}

Write-Host "Backup created: $BackupFile ($([math]::Round($BackupSize / 1MB, 2)) MB)" -ForegroundColor Green

# Create metadata file
Write-Host "`n[3/6] Creating metadata..." -ForegroundColor Yellow

$Metadata = @{
    backup_type = $BackupType
    timestamp = $Timestamp
    database_host = $DatabaseHost
    database_name = $DatabaseName
    backup_size_bytes = $BackupSize
    backup_file = (Split-Path -Leaf $BackupFile)
    kubernetes_namespace = $Namespace
    pod_name = $PodName
    checksum_sha256 = (Get-FileHash -Path $BackupFile -Algorithm SHA256).Hash
} | ConvertTo-Json -Depth 10

$Metadata | Out-File -FilePath $MetadataFile -Encoding UTF8
Write-Host "Metadata created: $MetadataFile" -ForegroundColor Green

# Upload to S3
Write-Host "`n[4/6] Uploading to S3..." -ForegroundColor Yellow

$S3BackupPath = "s3://$S3Bucket/$S3Prefix/$(Split-Path -Leaf $BackupFile)"
$S3MetadataPath = "s3://$S3Bucket/$S3Prefix/$(Split-Path -Leaf $MetadataFile)"

aws s3 cp $BackupFile $S3BackupPath --server-side-encryption AES256 --storage-class STANDARD_IA
aws s3 cp $MetadataFile $S3MetadataPath --server-side-encryption AES256 --storage-class STANDARD_IA

Write-Host "Uploaded to: $S3BackupPath" -ForegroundColor Green

# Verify upload
Write-Host "`n[5/6] Verifying upload..." -ForegroundColor Yellow

$S3FileSize = (aws s3 ls $S3BackupPath --summarize | Select-String "Total Size:" | ForEach-Object { $_.ToString().Split(":")[1].Trim() })

if ($S3FileSize -ne $BackupSize) {
    Write-Error "S3 file size ($S3FileSize) does not match local file size ($BackupSize)"
    exit 1
}

Write-Host "Upload verified successfully" -ForegroundColor Green

# Cleanup old local backups
Write-Host "`n[6/6] Cleaning up..." -ForegroundColor Yellow

Remove-Item -Path $BackupFile -Force
Remove-Item -Path $MetadataFile -Force

Write-Host "Local backups cleaned up" -ForegroundColor Green

# Report retention policy
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Backup completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "`nRetention Policy:" -ForegroundColor Yellow
Write-Host "  Full backups: 90 days" -ForegroundColor White
Write-Host "  Incremental backups: 7 days" -ForegroundColor White
Write-Host "`nTo restore this backup, run:" -ForegroundColor Yellow
Write-Host "  .\scripts\restore.ps1 -BackupFile $(Split-Path -Leaf $BackupFile)" -ForegroundColor White

exit 0
