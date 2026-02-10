# Production Certification

Based on [CERTIFICATE.md](../../CERTIFICATE.md) and [PRODUCTION_CERTIFICATION.md](../../PRODUCTION_CERTIFICATION.md).

## Current Status

❌ **NOT CERTIFIED FOR PRODUCTION**

See the complete [CERTIFICATE.md](../../CERTIFICATE.md) audit report for detailed findings.

## Critical Blockers

Before production deployment, these **must** be fixed:

### P0: Security & Correctness

1. **JWT revocation mechanism** - Compromised tokens valid until expiry
2. **Tunnel creation atomicity** - Crash mid-create leaves inconsistent state
3. **Tier limit enforcement** - Free users can create unlimited tunnels
4. **SSH brute force protection** - No rate limiting on SSH auth
5. **HTTP proxy timeouts** - Slow backends can exhaust connections
6. **Port validation** - Users can request reserved ports
7. **Config validation** - Invalid values cause runtime failures
8. **Frontend error handling** - Failed API calls show stale data
9. **Integration tests** - Breaking changes undetected
10. **Secrets in logs** - Potential credential leakage
11. **Dependency scanning** - Unknown CVEs in dependencies
12. **Metrics/monitoring** - Production issues invisible

## Certification Criteria

### Build & Startup Integrity

- [x] Deterministic builds
- [x] Missing env vars → hard fail
- [ ] Invalid config → hard fail (partially done)
- [ ] Database unreachable → explicit error
- [ ] Startup logs state version/config

### Configuration Discipline

- [x] No hardcoded domains/ports/limits/secrets
- [x] Safe defaults exist
- [ ] All tunables documented
- [ ] Secrets never logged
- [ ] Environment variables namespaced

### Authentication & Authorization

- [x] All protected routes require auth
- [x] JWTs signed, validated, expiring
- [ ] Token revocation possible
- [x] SSH requires valid identity
- [ ] Brute force rate limited
- [ ] Tier limits enforced

### Persistence & Data Integrity

- [x] No silent write failures
- [ ] Foreign keys enforced
- [ ] Migrations idempotent
- [ ] Schema matches models

### Security Hygiene

- [x] Secrets stored securely
- [x] No credentials in repo
- [ ] Dependency audit (`cargo audit`)
- [ ] Known vulnerabilities addressed

### Observability

- [x] Structured logs
- [x] Log levels used correctly
- [ ] No sensitive data logged
- [ ] Metrics exported (tunnels, auth, errors, latency)

### Operational Readiness

- [ ] Documented deploy process
- [ ] Rollback strategy
- [ ] On-call debugging feasible

## Recommendations by Priority

### High Priority (Blockers)

1. Add JWT revocation via Redis
2. Implement tier limit enforcement
3. Add HTTP proxy timeouts (10s default)
4. Validate config on startup
5. Add SSH auth rate limiting
6. Run `cargo audit` and fix CVEs
7. Add Prometheus metrics endpoint
8. Audit logs for sensitive data
9. Add integration test suite
10. Document deployment process

### Medium Priority

11. Make tunnel creation atomic
12. Add orphan tunnel cleanup
13. Complete `.env.example`
14. Frontend error state audit
15. Add startup version logging

### Low Priority

16. WebSocket rate limiting
17. Backpressure mechanisms
18. Slow client protection
19. Create operational runbooks

## Estimated Effort

**To certification**: 2-3 weeks

- Week 1: P0 security fixes
- Week 2: Integration tests + metrics
- Week 3: Documentation + audit

## Next Steps

1. Address P0 blockers (items 1-12 above)
2. Run security audit
3. Add integration tests
4. Implement metrics
5. Document operations
6. Re-audit with updated checklist
7. Load test before certification

## See Also

- [CERTIFICATE.md](../../CERTIFICATE.md) - Full audit report
- [PRODUCTION_CERTIFICATION.md](../../PRODUCTION_CERTIFICATION.md) - Certification requirements
- [Security](./security.md) - Security best practices
