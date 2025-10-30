# Disaster Recovery Documentation for dchat

## Overview

dchat implements comprehensive disaster recovery procedures to ensure data integrity and service continuity in the event of catastrophic failures.

**Recovery Objectives:**
- **RTO (Recovery Time Objective)**: < 1 hour
- **RPO (Recovery Point Objective)**: < 5 minutes

## Backup Strategy

### Full Backups
- **Frequency**: Daily at 2:00 AM UTC
- **Retention**: 90 days
- **Storage**: AWS S3 with encryption (AES-256)
- **Method**: `pg_dump` with custom format and compression

### Incremental Backups
- **Frequency**: Hourly
- **Retention**: 7 days
- **Storage**: AWS S3
- **Method**: PostgreSQL WAL (Write-Ahead Log) archiving

### Backup Verification
- **Frequency**: Daily
- **Checks**: File integrity, metadata verification, size validation
- **Alerts**: PagerDuty notification on verification failure

## Backup Operations

### Manual Backup

```powershell
# Full backup
.\scripts\backup.ps1 -BackupType full

# Incremental backup
.\scripts\backup.ps1 -BackupType incremental
```

### Automated Backups

Backups are automatically triggered by Kubernetes CronJobs:

```yaml
# Full backup CronJob (daily at 2 AM UTC)
apiVersion: batch/v1
kind: CronJob
metadata:
  name: dchat-backup-full
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: dchat/backup:latest
            command: ["pwsh", "-File", "/scripts/backup.ps1", "-BackupType", "full"]

# Incremental backup CronJob (hourly)
apiVersion: batch/v1
kind: CronJob
metadata:
  name: dchat-backup-incremental
spec:
  schedule: "0 * * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: dchat/backup:latest
            command: ["pwsh", "-File", "/scripts/backup.ps1", "-BackupType", "incremental"]
```

## Restore Procedures

### Full Database Restore

**Prerequisites:**
- Access to S3 backup bucket
- Kubernetes cluster access
- Database admin credentials

**Steps:**

1. **Identify backup to restore:**
   ```powershell
   aws s3 ls s3://$BACKUP_S3_BUCKET/backups/full/ --recursive
   ```

2. **Run restore script:**
   ```powershell
   .\scripts\restore.ps1 -BackupFile "dchat-full-2025-10-28_02-00-00.sql.gz"
   ```

3. **Verify restoration:**
   ```powershell
   kubectl exec -n default -it <pod-name> -- psql -h $DATABASE_HOST -U $DATABASE_USER -d $DATABASE_NAME -c "SELECT COUNT(*) FROM messages;"
   ```

4. **Monitor application health:**
   ```powershell
   kubectl logs -n default -l app.kubernetes.io/name=dchat -f
   ```

### Point-in-Time Recovery (PITR)

For recovery to a specific point in time using WAL archives:

1. **Restore latest full backup**
2. **Apply WAL archives up to target time**
3. **Set recovery target in postgresql.conf:**
   ```
   recovery_target_time = '2025-10-28 14:30:00'
   ```

## Disaster Scenarios

### Scenario 1: Database Corruption

**Symptoms:**
- PostgreSQL errors in logs
- Data inconsistencies
- Failed queries

**Recovery Steps:**
1. Stop all dchat pods to prevent writes
2. Assess corruption extent using `pg_check`
3. If repairable: Run `REINDEX` and `VACUUM FULL`
4. If not repairable: Restore from latest backup
5. Restart dchat pods
6. Verify data integrity

**Estimated Recovery Time:** 30-60 minutes

### Scenario 2: Complete Cluster Failure

**Symptoms:**
- Entire Kubernetes cluster unavailable
- All nodes unresponsive

**Recovery Steps:**
1. Provision new Kubernetes cluster using Terraform
2. Apply all infrastructure configurations
3. Deploy dchat using Helm charts
4. Restore database from S3 backup
5. Verify network connectivity
6. Update DNS to point to new cluster
7. Monitor application health

**Estimated Recovery Time:** 2-4 hours

### Scenario 3: Region-Wide Outage

**Symptoms:**
- Entire AWS region unavailable
- Multi-AZ failure

**Recovery Steps:**
1. Activate DR region (pre-configured standby)
2. Restore database from cross-region S3 replica
3. Update Route53 failover routing
4. Scale up DR cluster
5. Notify users of potential data loss (RPO limit)
6. Monitor replication lag

**Estimated Recovery Time:** 4-6 hours

### Scenario 4: Ransomware Attack

**Symptoms:**
- Encrypted files
- Suspicious process activity
- Ransom demands

**Recovery Steps:**
1. **Immediate Actions:**
   - Isolate affected systems
   - Disconnect network access
   - Preserve forensic evidence
   - Notify security team

2. **Assessment:**
   - Identify infection vector
   - Determine data impact
   - Check backup integrity

3. **Recovery:**
   - Provision clean infrastructure
   - Restore from verified clean backup
   - Implement additional security measures
   - Conduct security audit

**Estimated Recovery Time:** 6-12 hours

### Scenario 5: Data Center Failure

**Symptoms:**
- Multiple availability zones down
- Network partition

**Recovery Steps:**
1. Verify backup availability in S3
2. Provision infrastructure in alternate region
3. Deploy dchat stack
4. Restore database
5. Update DNS records
6. Verify application functionality

**Estimated Recovery Time:** 3-5 hours

## Backup Verification

### Daily Verification

Automated verification runs daily via CronJob:

```powershell
.\scripts\verify-backups.ps1 -DaysToCheck 7
```

### Manual Verification

Perform manual verification before major changes:

```powershell
# Verify specific backup
aws s3 cp s3://$BACKUP_S3_BUCKET/backups/full/dchat-full-2025-10-28_02-00-00.meta.json -
```

## Monitoring and Alerting

### Backup Alerts

- **Backup Failure**: Critical alert to PagerDuty
- **Verification Failure**: High severity alert
- **Old Backup**: Warning if no backup in 36 hours
- **Storage Growth**: Warning if S3 bucket > 80% quota

### Metrics

- `dchat_backup_last_success_timestamp`: Last successful backup
- `dchat_backup_duration_seconds`: Backup operation duration
- `dchat_backup_size_bytes`: Backup file size
- `dchat_restore_duration_seconds`: Restore operation duration

### Grafana Dashboard

Import the disaster recovery dashboard:

```bash
kubectl apply -f monitoring/dashboards/disaster-recovery-dashboard.json
```

## Testing

### Regular DR Drills

**Schedule:** Quarterly

**Procedure:**
1. Select random backup from past week
2. Provision test environment
3. Perform full restore
4. Verify data integrity
5. Load test restored environment
6. Document findings
7. Update procedures if needed

### Chaos Engineering

Run chaos tests monthly:

```bash
# Test pod failures
kubectl delete pod -n default -l app.kubernetes.io/name=dchat --random

# Test network partition
kubectl apply -f tests/chaos/network-partition.yaml

# Test database failover
kubectl exec -n default <rds-proxy-pod> -- kill -9 1
```

## Contact Information

### Escalation Path

1. **On-Call Engineer**: PagerDuty
2. **Database Team**: db-oncall@dchat.example.com
3. **Infrastructure Team**: infra-oncall@dchat.example.com
4. **Incident Commander**: +1-555-0100

### External Contacts

- **AWS Support**: Enterprise Support (24/7)
- **Database Vendor**: PostgreSQL Support
- **Security Team**: security@dchat.example.com

## Compliance

### Regulatory Requirements

- **GDPR**: 72-hour breach notification
- **SOC 2**: Annual audit of DR procedures
- **ISO 27001**: Documented DR testing

### Audit Trail

All restore operations are logged to:
- CloudWatch Logs
- S3 audit bucket
- Kubernetes audit logs

## Appendix

### Useful Commands

```powershell
# List all backups
aws s3 ls s3://$BACKUP_S3_BUCKET/backups/full/ --recursive --human-readable

# Check backup size trend
aws s3api list-objects --bucket $BACKUP_S3_BUCKET --prefix backups/full/ --query "reverse(sort_by(Contents, &LastModified))[0:10].[Key,Size,LastModified]" --output table

# Verify database connectivity
kubectl exec -n default -it <pod-name> -- psql -h $DATABASE_HOST -U $DATABASE_USER -d $DATABASE_NAME -c "SELECT version();"

# Check WAL archive status
kubectl exec -n default -it <pod-name> -- psql -h $DATABASE_HOST -U $DATABASE_USER -d $DATABASE_NAME -c "SELECT * FROM pg_stat_archiver;"
```

### Configuration Files

- Backup configuration: `scripts/backup.ps1`
- Restore configuration: `scripts/restore.ps1`
- Terraform state backup: `terraform/backend.tf`
- Database config: `helm/dchat/values.yaml`

### References

- [PostgreSQL Backup and Restore](https://www.postgresql.org/docs/current/backup.html)
- [AWS S3 Backup Best Practices](https://docs.aws.amazon.com/AmazonS3/latest/userguide/backup-best-practices.html)
- [Kubernetes Disaster Recovery](https://kubernetes.io/docs/tasks/run-application/configure-pdb/)
