#!/usr/bin/env pwsh
# Database Restore Script for dchat
# Restores database from S3 backup with verification

param(
    [Parameter(Mandatory=$true)]
    [string]$BackupFile,
    
    [Parameter(Mandatory=$false)]
    [string]$S3Bucket = $env:BACKUP_S3_BUCKET,
    
    [Parameter(Mandatory=$false)]
    [string]$DatabaseHost = $env:DATABASE_HOST,
    
    [Parameter(Mandatory=$false)]
    [string]$DatabaseName = $env:DATABASE_NAME,
    
    [Parameter(Mandatory=$false)]
    [string]$DatabaseUser = $env:DATABASE_USER,
    
    [Parameter(Mandatory=$false)]
    [string]$Namespace = "default",
    
    [Parameter(Mandatory=$false)]
    [switch]$Force
)

# Configuration
$ErrorActionPreference = "Stop"
$RestoreDir = "/tmp/restore"
$LocalBackupFile = "$RestoreDir/$BackupFile"
$S3BackupPath = "s3://$S3Bucket/backups/full/$BackupFile"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "dchat Database Restore" -ForegroundColor Cyan
Write-Host "Backup: $BackupFile" -ForegroundColor Cyan
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

# Warning prompt
if (-not $Force) {
    Write-Warning "This will REPLACE all data in database '$DatabaseName'!"
    $Confirmation = Read-Host "Type 'RESTORE' to continue"
    if ($Confirmation -ne "RESTORE") {
        Write-Host "Restore cancelled" -ForegroundColor Yellow
        exit 0
    }
}

# Create restore directory
New-Item -ItemType Directory -Force -Path $RestoreDir | Out-Null

# Get pod name
Write-Host "`n[1/7] Finding dchat pod..." -ForegroundColor Yellow
$PodName = kubectl get pods -n $Namespace -l app.kubernetes.io/name=dchat -o jsonpath='{.items[0].metadata.name}'

if (-not $PodName) {
    Write-Error "No dchat pod found in namespace $Namespace"
    exit 1
}
Write-Host "Found pod: $PodName" -ForegroundColor Green

# Download backup from S3
Write-Host "`n[2/7] Downloading backup from S3..." -ForegroundColor Yellow

aws s3 cp $S3BackupPath $LocalBackupFile

if (-not (Test-Path $LocalBackupFile)) {
    Write-Error "Failed to download backup from S3"
    exit 1
}

$BackupSize = (Get-Item $LocalBackupFile).Length
Write-Host "Downloaded: $LocalBackupFile ($([math]::Round($BackupSize / 1MB, 2)) MB)" -ForegroundColor Green

# Download and verify metadata
Write-Host "`n[3/7] Verifying backup integrity..." -ForegroundColor Yellow

$MetadataFile = $BackupFile -replace "\.sql\.gz$", ".meta.json"
$S3MetadataPath = "s3://$S3Bucket/backups/full/$MetadataFile"
$LocalMetadataFile = "$RestoreDir/$MetadataFile"

aws s3 cp $S3MetadataPath $LocalMetadataFile

if (Test-Path $LocalMetadataFile) {
    $Metadata = Get-Content $LocalMetadataFile | ConvertFrom-Json
    $ExpectedChecksum = $Metadata.checksum_sha256
    $ActualChecksum = (Get-FileHash -Path $LocalBackupFile -Algorithm SHA256).Hash
    
    if ($ExpectedChecksum -ne $ActualChecksum) {
        Write-Error "Checksum mismatch! Backup file may be corrupted."
        exit 1
    }
    
    Write-Host "Backup integrity verified" -ForegroundColor Green
} else {
    Write-Warning "Metadata file not found. Skipping checksum verification."
}

# Scale down dchat to prevent connections during restore
Write-Host "`n[4/7] Scaling down dchat deployment..." -ForegroundColor Yellow

$OriginalReplicas = kubectl get deployment -n $Namespace dchat -o jsonpath='{.spec.replicas}'
kubectl scale deployment -n $Namespace dchat --replicas=0
kubectl wait --for=delete pod -n $Namespace -l app.kubernetes.io/name=dchat --timeout=120s

Write-Host "dchat scaled down (was $OriginalReplicas replicas)" -ForegroundColor Green

# Terminate existing connections
Write-Host "`n[5/7] Terminating active database connections..." -ForegroundColor Yellow

$TerminateQuery = @"
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '$DatabaseName' AND pid <> pg_backend_pid();
"@

kubectl exec -n $Namespace $PodName -- bash -c "PGPASSWORD=`$DATABASE_PASSWORD psql -h $DatabaseHost -U $DatabaseUser -d postgres -c `"$TerminateQuery`""

Write-Host "Active connections terminated" -ForegroundColor Green

# Drop and recreate database
Write-Host "`n[6/7] Restoring database..." -ForegroundColor Yellow

kubectl exec -n $Namespace $PodName -- bash -c "PGPASSWORD=`$DATABASE_PASSWORD psql -h $DatabaseHost -U $DatabaseUser -d postgres -c `"DROP DATABASE IF EXISTS $DatabaseName;`""
kubectl exec -n $Namespace $PodName -- bash -c "PGPASSWORD=`$DATABASE_PASSWORD psql -h $DatabaseHost -U $DatabaseUser -d postgres -c `"CREATE DATABASE $DatabaseName;`""

# Restore from backup
kubectl exec -n $Namespace $PodName -- bash -c "PGPASSWORD=`$DATABASE_PASSWORD pg_restore -h $DatabaseHost -U $DatabaseUser -d $DatabaseName --clean --if-exists --no-owner --no-acl" < $LocalBackupFile

Write-Host "Database restored" -ForegroundColor Green

# Scale dchat back up
Write-Host "`n[7/7] Scaling dchat back up..." -ForegroundColor Yellow

kubectl scale deployment -n $Namespace dchat --replicas=$OriginalReplicas
kubectl wait --for=condition=ready pod -n $Namespace -l app.kubernetes.io/name=dchat --timeout=180s

Write-Host "dchat scaled back up to $OriginalReplicas replicas" -ForegroundColor Green

# Cleanup
Remove-Item -Path $LocalBackupFile -Force
Remove-Item -Path $LocalMetadataFile -Force -ErrorAction SilentlyContinue

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Restore completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "`nVerify the restore:" -ForegroundColor Yellow
Write-Host "  kubectl exec -n $Namespace -it <pod-name> -- psql -h $DatabaseHost -U $DatabaseUser -d $DatabaseName -c 'SELECT COUNT(*) FROM messages;'" -ForegroundColor White

exit 0
