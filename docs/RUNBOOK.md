# Meridian Operations Runbook

**Version:** 1.0
**Last Updated:** December 30, 2025
**Classification:** Internal Operations

---

## Table of Contents

1. [Service Overview](#service-overview)
2. [Health Checks & Monitoring](#health-checks--monitoring)
3. [Incident Response Procedures](#incident-response-procedures)
4. [Circuit Breaker Operations](#circuit-breaker-operations)
5. [Database Operations](#database-operations)
6. [Secret Rotation](#secret-rotation)
7. [Emergency Procedures](#emergency-procedures)
8. [Contact Information](#contact-information)

---

## Service Overview

### Architecture

```
                    ┌─────────────────┐
                    │   Load Balancer │
                    │   (Railway/ALB) │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
       ┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼─────┐
       │ API Pod 1   │ │ API Pod 2 │ │ API Pod N │
       │ meridian-api│ │           │ │           │
       └──────┬──────┘ └─────┬─────┘ └─────┬─────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
       ┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼─────┐
       │  PostgreSQL │ │  Ethereum │ │   Oracle  │
       │   (RDS)     │ │    RPC    │ │  Chainlink│
       └─────────────┘ └───────────┘ └───────────┘
```

### Key Services

| Service | Purpose | Port | Health Endpoint |
|---------|---------|------|-----------------|
| meridian-api | REST API Server | 8080 | `/health` |
| PostgreSQL | Primary Database | 5432 | pg_isready |
| Ethereum RPC | Blockchain Access | 8545 | eth_blockNumber |
| Frontend | Next.js Dashboard | 3000 | `/api/health` |

### Environment Variables

See `production_deployment.md` for complete list. Critical variables:

```bash
DATABASE_URL          # PostgreSQL connection string
ETHEREUM_RPC_URL      # Ethereum node endpoint
SESSION_TOKEN_SALT    # 32+ byte secret for token hashing
API_KEY_SALT          # 32+ byte secret for API key hashing
JWT_SECRET            # JWT signing secret
```

---

## Health Checks & Monitoring

### Health Endpoint Response

```bash
# Check API health
curl -s http://api.meridian.finance/health | jq .
```

**Healthy Response:**
```json
{
  "status": "ok",
  "version": "0.1.0",
  "oracle_enabled": true,
  "baskets_count": 5
}
```

**Unhealthy Response:**
```json
{
  "status": "unhealthy",
  "error": "database connection failed",
  "version": "0.1.0"
}
```

### Prometheus Metrics

```bash
# Fetch metrics (requires admin auth)
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://api.meridian.finance/metrics
```

Key metrics to monitor:
- `meridian_db_pool_size` - Database connection pool size
- `meridian_db_pool_idle` - Idle database connections
- `meridian_oracle_enabled` - Oracle integration status
- `meridian_baskets_total` - Total number of baskets
- `meridian_users_total` - Total registered users
- `meridian_operations_total{type="mint|burn"}` - Operation counts

### OpenTelemetry Tracing

Traces are exported via OTLP when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.

```bash
# Enable tracing to Jaeger
export OTEL_EXPORTER_OTLP_ENDPOINT="http://jaeger:4317"
export OTEL_SERVICE_NAME="meridian-api"
export OTEL_TRACES_SAMPLER_ARG="0.1"  # 10% sampling
```

### Log Aggregation

Logs are structured JSON when `LOG_FORMAT=json`:

```json
{
  "timestamp": "2025-12-30T15:30:00Z",
  "level": "INFO",
  "target": "meridian_api",
  "message": "Request completed",
  "correlation_id": "abc-123-def",
  "duration_ms": 45
}
```

---

## Incident Response Procedures

### INC-001: API Unresponsive

**Symptoms:**
- `/health` returns 5xx or times out
- Increased error rates in monitoring
- User reports of failed transactions

**Investigation:**
```bash
# 1. Check pod status
kubectl get pods -n meridian

# 2. Check pod logs
kubectl logs -n meridian deployment/meridian-api --tail=100

# 3. Check database connectivity
kubectl exec -it meridian-api-xxx -- curl -s localhost:8080/health

# 4. Check database pool exhaustion
grep "pool exhausted" logs | tail -20
```

**Resolution:**
1. If database pool exhausted: Restart pods to clear connections
2. If memory pressure: Scale up pods or increase memory limits
3. If external dependency failure: Check circuit breaker status

### INC-002: Database Pool Exhaustion

**Symptoms:**
- `/health` returns `database connection failed`
- Logs show `pool exhausted` errors
- Slow response times across all endpoints

**Investigation:**
```bash
# Check active connections
psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity;"

# Check connection state
psql $DATABASE_URL -c "SELECT state, count(*) FROM pg_stat_activity GROUP BY state;"

# Check for long-running queries
psql $DATABASE_URL -c "SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE state != 'idle'
ORDER BY duration DESC
LIMIT 10;"
```

**Resolution:**
1. Kill long-running queries if blocking
2. Increase pool size in configuration
3. Rolling restart of API pods
4. If persistent, investigate query patterns

### INC-003: Oracle RPC Failure

**Symptoms:**
- Circuit breaker open for oracle
- Price feed updates failing
- Mint/burn operations returning errors

**Investigation:**
```bash
# Check RPC endpoint health
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  $ETHEREUM_RPC_URL

# Check circuit breaker state in logs
grep "circuit_breaker" logs | tail -20
```

**Resolution:**
1. If RPC provider issue: Switch to backup RPC endpoint
2. If rate limited: Reduce request rate, contact provider
3. If network issue: Wait for automatic recovery (30s cooldown)

### INC-004: Rate Limiting Active

**Symptoms:**
- Users receiving 429 Too Many Requests
- Logs show rate limit exceeded

**Investigation:**
```bash
# Check rate limit headers in response
curl -v http://api.meridian.finance/health 2>&1 | grep -i "x-ratelimit"
```

**Resolution:**
1. If legitimate traffic spike: Temporarily increase rate limits
2. If attack: Block offending IPs at load balancer
3. Review rate limit configuration in routes.rs

---

## Circuit Breaker Operations

### Circuit Breaker States

| State | Description | Behavior |
|-------|-------------|----------|
| Closed | Normal operation | All requests pass through |
| Open | Failing fast | Requests immediately return error |
| Half-Open | Testing recovery | Limited requests allowed to test |

### Configuration

```rust
// Current settings (state.rs)
const FAILURE_THRESHOLD: u32 = 5;     // Opens after 5 failures
const RECOVERY_TIMEOUT: Duration = 30s; // Wait before half-open
const SUCCESS_THRESHOLD: u32 = 2;     // Closes after 2 successes
```

### Manual Circuit Breaker Reset

**Note:** There is no manual reset endpoint. Circuit breakers auto-recover.

To force reset:
1. Rolling restart of API pods
2. Or wait for recovery timeout (30 seconds)

### Monitoring Circuit Breaker

```bash
# Check logs for circuit state changes
grep "circuit" logs | grep -E "open|closed|half"
```

---

## Database Operations

### Connection String Format

```
postgresql://user:password@host:5432/meridian_prod?sslmode=require
```

### Running Migrations

```bash
# From crates/db directory
export DATABASE_URL="postgresql://..."
sqlx migrate run

# Verify migrations
sqlx migrate info
```

### Backup Strategy

**CRITICAL: Financial Platform Backup Requirements**

| Metric | Target | Rationale |
|--------|--------|-----------|
| RPO (Recovery Point Objective) | 1 hour | Maximum acceptable data loss for financial transactions |
| RTO (Recovery Time Objective) | 4 hours | Maximum acceptable downtime for recovery |
| Retention Period | 90 days | Regulatory compliance requirement |
| Backup Testing | Weekly | Verify restore procedures work |

### Automated Backup Configuration

**Railway PostgreSQL:**
```bash
# Railway automatically creates daily backups
# Retention: 7 days (default), extend via Railway dashboard

# To verify backup status:
railway logs --json | grep "backup"
```

**AWS RDS (Production):**
```bash
# Configure automated backups
aws rds modify-db-instance \
  --db-instance-identifier meridian-prod \
  --backup-retention-period 90 \
  --preferred-backup-window "03:00-04:00" \
  --enable-cloudwatch-logs-exports '["postgresql"]' \
  --apply-immediately
```

**Backup Schedule:**
| Type | Frequency | Retention | Storage |
|------|-----------|-----------|---------|
| Automated Snapshot | Daily 03:00 UTC | 90 days | RDS/S3 |
| Transaction Logs | Continuous | 7 days | RDS WAL |
| Manual Snapshot | Pre-deployment | 30 days | S3 |
| Off-site Archive | Weekly | 1 year | AWS Glacier |

### Manual Backup Procedures

**Full Database Backup:**
```bash
#!/bin/bash
# backup_database.sh - Run before any major changes

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="meridian_backup_${DATE}.sql"
S3_BUCKET="meridian-backups-prod"

# Create backup with compression
pg_dump $DATABASE_URL \
  --format=custom \
  --compress=9 \
  --file="${BACKUP_FILE}.gz" \
  --verbose

# Verify backup integrity
pg_restore --list "${BACKUP_FILE}.gz" > /dev/null
if [ $? -eq 0 ]; then
  echo "Backup verified successfully"

  # Upload to S3
  aws s3 cp "${BACKUP_FILE}.gz" "s3://${S3_BUCKET}/daily/${BACKUP_FILE}.gz"

  # Calculate and store checksum
  sha256sum "${BACKUP_FILE}.gz" > "${BACKUP_FILE}.gz.sha256"
  aws s3 cp "${BACKUP_FILE}.gz.sha256" "s3://${S3_BUCKET}/daily/${BACKUP_FILE}.gz.sha256"
else
  echo "ERROR: Backup verification failed!"
  exit 1
fi
```

**Critical Tables Backup (for quick restore):**
```bash
# Backup only critical financial tables
pg_dump $DATABASE_URL \
  --table=stablecoins \
  --table=baskets \
  --table=operations \
  --table=audit_logs \
  --format=custom \
  --file="meridian_critical_$(date +%Y%m%d).sql.gz"
```

### Point-in-Time Recovery (PITR)

**AWS RDS - Restore to Specific Time:**
```bash
# 1. Identify the target restore time
TARGET_TIME="2025-12-30T12:00:00Z"

# 2. Create restored instance
aws rds restore-db-instance-to-point-in-time \
  --source-db-instance-identifier meridian-prod \
  --target-db-instance-identifier meridian-prod-restored \
  --restore-time "${TARGET_TIME}" \
  --db-instance-class db.t3.medium \
  --vpc-security-group-ids sg-xxxx

# 3. Wait for instance to be available
aws rds wait db-instance-available \
  --db-instance-identifier meridian-prod-restored

# 4. Verify data integrity
psql $RESTORED_DATABASE_URL -c "SELECT COUNT(*) FROM operations WHERE created_at > '${TARGET_TIME}';"

# 5. If verified, update application to point to restored instance
# Update DATABASE_URL in Railway/environment variables
```

### Restore Procedures

**Full Restore from Backup:**
```bash
#!/bin/bash
# restore_database.sh

BACKUP_FILE=$1
TARGET_DB="meridian_restored"

# 1. Verify backup checksum
aws s3 cp "s3://meridian-backups-prod/daily/${BACKUP_FILE}.sha256" .
sha256sum -c "${BACKUP_FILE}.sha256"
if [ $? -ne 0 ]; then
  echo "ERROR: Checksum verification failed!"
  exit 1
fi

# 2. Create new database
psql $DATABASE_URL -c "CREATE DATABASE ${TARGET_DB};"

# 3. Restore from backup
pg_restore \
  --dbname="${TARGET_DB}" \
  --verbose \
  --no-owner \
  --no-privileges \
  "${BACKUP_FILE}"

# 4. Verify restore
psql "${TARGET_DB}" -c "SELECT COUNT(*) FROM baskets;"
psql "${TARGET_DB}" -c "SELECT COUNT(*) FROM operations;"
psql "${TARGET_DB}" -c "SELECT MAX(created_at) FROM audit_logs;"

# 5. Run data integrity checks
echo "Verify audit_logs immutability triggers exist:"
psql "${TARGET_DB}" -c "\d audit_logs"

echo "Verify foreign key constraints:"
psql "${TARGET_DB}" -c "SELECT * FROM pg_constraint WHERE contype = 'f';"
```

**Selective Table Restore:**
```bash
# Restore specific tables without affecting others
pg_restore \
  --dbname=$DATABASE_URL \
  --table=operations \
  --data-only \
  --disable-triggers \
  "${BACKUP_FILE}"
```

### Backup Verification (Weekly)

**Automated Backup Test Script:**
```bash
#!/bin/bash
# test_backup_restore.sh - Run weekly via cron

set -e

# 1. Get latest backup
LATEST=$(aws s3 ls s3://meridian-backups-prod/daily/ --recursive | sort | tail -1 | awk '{print $4}')
aws s3 cp "s3://meridian-backups-prod/${LATEST}" /tmp/test_restore.sql.gz

# 2. Create test database
TEST_DB="meridian_backup_test_$(date +%Y%m%d)"
psql $DATABASE_URL -c "CREATE DATABASE ${TEST_DB};"

# 3. Restore
pg_restore --dbname="${TEST_DB}" /tmp/test_restore.sql.gz

# 4. Run integrity checks
RESULT=$(psql ${TEST_DB} -t -c "
  SELECT
    CASE
      WHEN (SELECT COUNT(*) FROM baskets) > 0
       AND (SELECT COUNT(*) FROM stablecoins) >= 0
       AND (SELECT COUNT(*) FROM audit_logs) > 0
      THEN 'PASS'
      ELSE 'FAIL'
    END;
")

# 5. Cleanup
psql $DATABASE_URL -c "DROP DATABASE ${TEST_DB};"
rm /tmp/test_restore.sql.gz

# 6. Report
if [[ "$RESULT" == *"PASS"* ]]; then
  echo "Backup verification PASSED - $(date)"
  # Send success notification
  curl -X POST $SLACK_WEBHOOK -d '{"text":"Weekly backup verification PASSED"}'
else
  echo "Backup verification FAILED - $(date)"
  # Send alert
  curl -X POST $PAGERDUTY_WEBHOOK -d '{"event_type":"trigger","description":"Backup verification failed"}'
  exit 1
fi
```

### Disaster Recovery Scenarios

**Scenario 1: Single Table Corruption**
1. Identify corrupted table and time of corruption
2. Create PITR restore to pre-corruption time
3. Export clean table data
4. Restore table to production
5. Replay valid transactions from audit_log

**Scenario 2: Complete Database Loss**
1. Activate incident response (INC-001)
2. Provision new RDS instance
3. Restore from latest verified backup
4. Apply transaction logs up to failure point
5. Verify data integrity
6. Update application configuration
7. Verify health checks
8. Resume operations

**Scenario 3: Region Failure**
1. Activate DR plan
2. Promote read replica in secondary region
3. Update DNS to point to secondary region
4. Verify all services operational
5. Begin cross-region data sync once primary recovers

### Backup Monitoring Alerts

```yaml
# Prometheus alerting rules for backups
groups:
  - name: backup_alerts
    rules:
      - alert: BackupNotCompletedIn24Hours
        expr: time() - backup_last_success_timestamp > 86400
        for: 1h
        labels:
          severity: critical
        annotations:
          summary: "Database backup has not completed in 24 hours"

      - alert: BackupSizeTooSmall
        expr: backup_size_bytes < 1000000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Backup file is suspiciously small"

      - alert: BackupVerificationFailed
        expr: backup_verification_success == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Weekly backup verification failed"
```

### Schema Changes

1. Create migration: `sqlx migrate add <name>`
2. Test locally: `sqlx migrate run` on dev database
3. Deploy to staging
4. Deploy to production during maintenance window

---

## Secret Rotation

### JWT_SECRET Rotation

**Impact:** All active sessions will be invalidated

**Procedure:**
1. Generate new 256-bit secret: `openssl rand -base64 32`
2. Update Railway/Vercel environment variable
3. Trigger rolling deployment
4. Monitor for authentication errors

### SESSION_TOKEN_SALT Rotation

**Impact:** All stored session tokens become invalid

**Procedure:**
1. Generate new 32-byte salt: `openssl rand -hex 16`
2. Update environment variable
3. Trigger rolling deployment
4. All users will need to re-authenticate

### API_KEY_SALT Rotation

**Impact:** All agent API keys become invalid

**Procedure:**
1. Notify affected agent operators (24h notice)
2. Generate new salt: `openssl rand -hex 16`
3. Update environment variable
4. Deploy during maintenance window
5. Agents must regenerate API keys

### Database Credentials Rotation

**Procedure:**
1. Create new database user with same permissions
2. Update DATABASE_URL in environment
3. Trigger rolling deployment
4. Verify health checks pass
5. Revoke old database user credentials

---

## Emergency Procedures

### EMERGENCY: Production Down

**Immediate Actions (within 5 minutes):**
1. Check load balancer status
2. Check pod health: `kubectl get pods -n meridian`
3. Check recent deployments: `kubectl rollout history deployment/meridian-api`
4. If recent deployment: Rollback immediately

**Rollback Procedure:**
```bash
# Kubernetes
kubectl rollout undo deployment/meridian-api -n meridian

# Railway
railway rollback

# Verify
kubectl rollout status deployment/meridian-api -n meridian
```

### EMERGENCY: Security Breach Suspected

**Immediate Actions:**
1. Rotate all secrets (JWT_SECRET, salts)
2. Invalidate all sessions (database: `TRUNCATE sessions;`)
3. Enable enhanced logging
4. Review audit logs for suspicious activity
5. Contact security team

### EMERGENCY: Database Corruption

**Immediate Actions:**
1. Stop all API pods (prevent further damage)
2. Create snapshot of current state
3. Initiate PITR to last known good state
4. Notify stakeholders

### EMERGENCY: Smart Contract Issue

**Immediate Actions:**
1. Pause contract if pauseable: Call `pause()` function
2. Notify all integrated services
3. Investigate root cause
4. Plan remediation (upgrade if UUPS proxy)

### Emergency Contacts

| Role | Contact | Escalation |
|------|---------|------------|
| On-Call Engineer | PagerDuty | Immediate |
| Backend Lead | [Email] | 5 minutes |
| Security Lead | [Email] | Security issues |
| Database Admin | [Email] | Database issues |

---

## Appendix: Common Commands

```bash
# API Health Check
curl http://api.meridian.finance/health

# View recent logs
kubectl logs -f deployment/meridian-api -n meridian

# Scale API pods
kubectl scale deployment/meridian-api --replicas=5 -n meridian

# Database connection test
psql $DATABASE_URL -c "SELECT NOW();"

# Check Ethereum RPC
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  $ETHEREUM_RPC_URL

# Generate new secrets
openssl rand -base64 32  # JWT secret
openssl rand -hex 16     # 32-byte salt
```

---

*Document maintained by: Platform Engineering Team*
*Last reviewed: December 30, 2025*
