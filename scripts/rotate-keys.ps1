#!/usr/bin/env pwsh
# Automated Key Rotation Script for dchat
# Rotates cryptographic keys every 90 days

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("validator", "database", "api", "all")]
    [string]$KeyType = "all",
    
    [Parameter(Mandatory=$false)]
    [string]$Environment = $env:DCHAT_ENVIRONMENT,
    
    [Parameter(Mandatory=$false)]
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "dchat Key Rotation" -ForegroundColor Cyan
Write-Host "Environment: $Environment" -ForegroundColor Cyan
Write-Host "Key Type: $KeyType" -ForegroundColor Cyan
Write-Host "Dry Run: $DryRun" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Validate environment
if (-not $Environment) {
    Write-Error "Environment not specified. Set DCHAT_ENVIRONMENT or use -Environment parameter."
    exit 1
}

# Rotation functions

function Rotate-ValidatorKey {
    Write-Host "`n[1/5] Rotating validator signing key..." -ForegroundColor Yellow
    
    # Get current key ID from KMS
    $CurrentKeyId = aws kms describe-key --key-id "alias/dchat-validator-$Environment" --query 'KeyMetadata.KeyId' --output text
    
    if (-not $CurrentKeyId) {
        Write-Error "Current validator key not found"
        return $false
    }
    
    Write-Host "Current key ID: $CurrentKeyId" -ForegroundColor White
    
    # Create new key
    if (-not $DryRun) {
        Write-Host "Creating new validator key..." -ForegroundColor White
        
        $NewKey = aws kms create-key `
            --description "dchat validator signing key - $(Get-Date -Format 'yyyy-MM-dd')" `
            --key-usage SIGN_VERIFY `
            --customer-master-key-spec ECC_SECG_P256K1 `
            --tags "TagKey=Environment,TagValue=$Environment" "TagKey=Purpose,TagValue=validator-signing" `
            --output json | ConvertFrom-Json
        
        $NewKeyId = $NewKey.KeyMetadata.KeyId
        Write-Host "New key created: $NewKeyId" -ForegroundColor Green
        
        # Update alias to point to new key
        aws kms update-alias --alias-name "alias/dchat-validator-$Environment" --target-key-id $NewKeyId
        Write-Host "Alias updated to new key" -ForegroundColor Green
        
        # Schedule old key for deletion (30 days)
        aws kms schedule-key-deletion --key-id $CurrentKeyId --pending-window-in-days 30
        Write-Host "Old key scheduled for deletion in 30 days" -ForegroundColor Yellow
        
        # Update Kubernetes secret with new key reference
        kubectl create secret generic dchat-validator-key `
            --from-literal=key-id=$NewKeyId `
            --namespace=default `
            --dry-run=client -o yaml | kubectl apply -f -
        
        Write-Host "Kubernetes secret updated" -ForegroundColor Green
    } else {
        Write-Host "[DRY RUN] Would rotate validator key $CurrentKeyId" -ForegroundColor Yellow
    }
    
    return $true
}

function Rotate-DatabaseCredentials {
    Write-Host "`n[2/5] Rotating database credentials..." -ForegroundColor Yellow
    
    # Get current secret
    $SecretArn = "dchat/database/$Environment"
    $CurrentSecret = aws secretsmanager get-secret-value --secret-id $SecretArn --query 'SecretString' --output text | ConvertFrom-Json
    
    if (-not $CurrentSecret) {
        Write-Error "Current database secret not found"
        return $false
    }
    
    Write-Host "Current username: $($CurrentSecret.username)" -ForegroundColor White
    
    if (-not $DryRun) {
        # Generate new password
        $NewPassword = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 32 | ForEach-Object {[char]$_})
        
        # Update RDS password
        Write-Host "Updating RDS password..." -ForegroundColor White
        aws rds modify-db-instance `
            --db-instance-identifier "dchat-$Environment" `
            --master-user-password $NewPassword `
            --apply-immediately
        
        # Wait for password change to complete
        Write-Host "Waiting for password change..." -ForegroundColor White
        Start-Sleep -Seconds 30
        
        # Update Secrets Manager
        $NewSecret = @{
            username = $CurrentSecret.username
            password = $NewPassword
            host     = $CurrentSecret.host
            port     = $CurrentSecret.port
            database = $CurrentSecret.database
        } | ConvertTo-Json
        
        aws secretsmanager put-secret-value --secret-id $SecretArn --secret-string $NewSecret
        Write-Host "Secrets Manager updated" -ForegroundColor Green
        
        # Restart dchat pods to pick up new credentials
        Write-Host "Restarting dchat pods..." -ForegroundColor White
        kubectl rollout restart deployment/dchat -n default
        kubectl rollout status deployment/dchat -n default --timeout=5m
        
        Write-Host "Pods restarted successfully" -ForegroundColor Green
    } else {
        Write-Host "[DRY RUN] Would rotate database password" -ForegroundColor Yellow
    }
    
    return $true
}

function Rotate-ApiKeys {
    Write-Host "`n[3/5] Rotating API keys..." -ForegroundColor Yellow
    
    $SecretArn = "dchat/api-keys/$Environment"
    
    if (-not $DryRun) {
        # Generate new API keys
        $NewApiKey = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 64 | ForEach-Object {[char]$_})
        $NewWebhookSecret = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 64 | ForEach-Object {[char]$_})
        
        $NewSecret = @{
            api_key        = $NewApiKey
            webhook_secret = $NewWebhookSecret
            rotated_at     = (Get-Date).ToUniversalTime().ToString("o")
        } | ConvertTo-Json
        
        aws secretsmanager put-secret-value --secret-id $SecretArn --secret-string $NewSecret
        Write-Host "API keys rotated" -ForegroundColor Green
        
        # Update ConfigMap
        kubectl create configmap dchat-api-config `
            --from-literal=api-key-version="$(Get-Date -Format 'yyyyMMddHHmmss')" `
            --namespace=default `
            --dry-run=client -o yaml | kubectl apply -f -
        
        Write-Host "ConfigMap updated" -ForegroundColor Green
    } else {
        Write-Host "[DRY RUN] Would rotate API keys" -ForegroundColor Yellow
    }
    
    return $true
}

function Update-TlsCertificates {
    Write-Host "`n[4/5] Checking TLS certificates..." -ForegroundColor Yellow
    
    # Get certificate expiration
    $CertArn = aws acm list-certificates --query "CertificateSummaryList[?DomainName=='dchat.example.com'].CertificateArn" --output text
    
    if ($CertArn) {
        $CertDetails = aws acm describe-certificate --certificate-arn $CertArn --output json | ConvertFrom-Json
        $Expiry = [DateTime]::Parse($CertDetails.Certificate.NotAfter)
        $DaysUntilExpiry = ($Expiry - (Get-Date)).Days
        
        Write-Host "Certificate expires in $DaysUntilExpiry days ($Expiry)" -ForegroundColor White
        
        if ($DaysUntilExpiry -lt 30) {
            Write-Warning "Certificate expires soon! Consider renewing."
            
            if (-not $DryRun) {
                Write-Host "Triggering certificate renewal..." -ForegroundColor White
                # ACM auto-renews, but we can force validation
                aws acm request-certificate-validation --certificate-arn $CertArn
            }
        } else {
            Write-Host "Certificate is valid" -ForegroundColor Green
        }
    } else {
        Write-Warning "No certificate found for dchat.example.com"
    }
    
    return $true
}

function Send-RotationNotification {
    Write-Host "`n[5/5] Sending rotation notification..." -ForegroundColor Yellow
    
    $Message = @"
dchat Key Rotation Completed
Environment: $Environment
Key Type: $KeyType
Timestamp: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss UTC")
Status: Success

Keys rotated:
- Validator signing key
- Database credentials
- API keys

Next rotation: $(Get-Date).AddDays(90).ToString("yyyy-MM-dd")
"@
    
    if (-not $DryRun) {
        # Send to Slack
        $SlackWebhook = $env:SLACK_WEBHOOK_URL
        if ($SlackWebhook) {
            $Payload = @{
                text = $Message
            } | ConvertTo-Json
            
            Invoke-RestMethod -Uri $SlackWebhook -Method Post -Body $Payload -ContentType 'application/json'
            Write-Host "Slack notification sent" -ForegroundColor Green
        }
        
        # Log to CloudWatch
        aws logs put-log-events `
            --log-group-name "/dchat/security/key-rotation" `
            --log-stream-name "$Environment-$(Get-Date -Format 'yyyy-MM-dd')" `
            --log-events "timestamp=$(Get-Date -UFormat %s),message=$Message"
        
        Write-Host "CloudWatch log written" -ForegroundColor Green
    } else {
        Write-Host "[DRY RUN] Would send notification" -ForegroundColor Yellow
    }
    
    return $true
}

# Main rotation logic

try {
    $Success = $true
    
    if ($KeyType -eq "validator" -or $KeyType -eq "all") {
        $Success = $Success -and (Rotate-ValidatorKey)
    }
    
    if ($KeyType -eq "database" -or $KeyType -eq "all") {
        $Success = $Success -and (Rotate-DatabaseCredentials)
    }
    
    if ($KeyType -eq "api" -or $KeyType -eq "all") {
        $Success = $Success -and (Rotate-ApiKeys)
    }
    
    if ($KeyType -eq "all") {
        $Success = $Success -and (Update-TlsCertificates)
        $Success = $Success -and (Send-RotationNotification)
    }
    
    Write-Host "`n========================================" -ForegroundColor Cyan
    if ($Success) {
        Write-Host "Key rotation completed successfully!" -ForegroundColor Green
    } else {
        Write-Host "Key rotation completed with warnings" -ForegroundColor Yellow
    }
    Write-Host "========================================" -ForegroundColor Cyan
    
    if ($DryRun) {
        Write-Host "`nThis was a dry run. No changes were made." -ForegroundColor Yellow
        Write-Host "Run without -DryRun to perform actual rotation." -ForegroundColor Yellow
    }
    
    exit 0
    
} catch {
    Write-Error "Key rotation failed: $_"
    
    # Send failure notification
    if (-not $DryRun -and $env:SLACK_WEBHOOK_URL) {
        $ErrorPayload = @{
            text = "❌ dchat Key Rotation FAILED`nEnvironment: $Environment`nError: $_"
        } | ConvertTo-Json
        
        Invoke-RestMethod -Uri $env:SLACK_WEBHOOK_URL -Method Post -Body $ErrorPayload -ContentType 'application/json'
    }
    
    exit 1
}
