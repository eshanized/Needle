# Dependency Security Audit Results

## Summary

**Date:** 2026-02-10  
**Tool:** cargo-audit v0.20.0  
**Findings:** 2 vulnerabilities

---

## Vulnerabilities Found

### 1. RUSTSEC-2024-0437 - protobuf (via prometheus)

**Severity:** Not specified (likely MEDIUM)  
**Crate:** `protobuf 2.28.0`  
**Title:** Crash due to uncontrolled recursion in protobuf crate  
**Date:** 2024-12-12  
**URL:** https://rustsec.org/advisories/RUSTSEC-2024-0437

**Dependency Path:**
```
protobuf 2.28.0
└── prometheus 0.13.4
    ├── needle-core 0.1.0
    └── needle-api 0.1.0
```

**Solution:** Upgrade prometheus to version using protobuf >= 3.7.2

**Risk Assessment:**
- **Impact:** Denial of Service (crash)
- **Exploitability:** Low - requires maliciously crafted protobuf data  
- **Our Exposure:** Minimal - prometheus usage is metrics export only, no external protobuf parsing

**Mitigation Plan:**
```bash
# Update prometheus dependency
# In Cargo.toml workspace dependencies:
prometheus = "0.14"  # Latest version uses protobuf 3.x
```

---

### 2. RUSTSEC-2023-0071 - rsa (via russh)

**Severity:** 5.9 (MEDIUM)  
**Crate:** `rsa 0.9.10`  
**Title:** Marvin Attack: potential key recovery through timing sidechannels  
**Date:** 2023-11-22  
**URL:** https://rustsec.org/advisories/RUSTSEC-2023-0071

**Dependency Path:**
```
rsa 0.9.10
├── ssh-key 0.6.7
│   └── russh-keys 0.46.0
│       └── russh 0.46.0
│           └── needle-core 0.1.0
```

**Solution:** No fixed upgrade available (as of audit date)

**Risk Assessment:**
- **Impact:** Private key recovery via timing attack
- **Exploitability:** High skill required, network timing measurement needed
- **Our Exposure:** Low - SSH keys used for client authentication only, not server private keys

**Mitigation Options:**

1. **Accept Risk** (Recommended for now)
   - SSH client keys are short-lived
   - Attack requires precise timing measurements
   - Server does not use RSA for its own authentication
   - Monitor russh updates for patch

2. **Disable RSA** (If possible)
   - Check if russh supports ED25519-only mode
   - Would eliminate RSA timing channel entirely

3. **Network Isolation**
   - Ensure SSH port not exposed to untrusted networks
   - Use firewall rules to limit access

**Accepted Risk Justification:**
```
The RSA timing vulnerability affects SSH client key operations, not server operations.
Our threat model prioritizes availability and functionality over this specific attack vector.
The attack requires sophisticated timing measurements and high-precision networking.
We will monitor russh releases for upstream fixes and update when available.
```

---

## Action Items

### Immediate (P0)
- [ ] Update prometheus to 0.14.0 to fix protobuf CVE
- [ ] Test metrics endpoint still works after prometheus upgrade
- [ ] Re-run `cargo audit` to confirm fix

### Short-term (P1)
- [ ] Monitor russh project for RSA vulnerability fixes
- [ ] Consider disabling RSA key support if ED25519 sufficient
- [ ] Add `cargo audit` to CI pipeline to catch future CVEs

### Long-term (P2)
- [ ] Document accepted risks in security policy
- [ ] Periodic dependency updates (monthly)
- [ ] Security advisory monitoring

---

## CI Integration

Add to `.github/workflows/security.yml` (or .gitlab-ci.yml):

```yaml
name: Security Audit

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/audit@v1
        with:
          # Fail build on HIGH/CRITICAL only
          deny: high
```

---

## Fix Commands

```bash
# 1. Update prometheus dependency
cd /home/snigdha/Needle/libneedle
# Edit Cargo.toml workspace section:
# prometheus = "0.14"

# 2. Update dependencies
cargo update -p prometheus

# 3. Verify fix
cargo audit

# 4. Test compilation
cargo check --workspace

# 5. Test metrics endpoint
cargo run --bin needle-server &
sleep 2
curl http://localhost:3000/metrics
pkill needle-server
```

---

## Security Policy

**CVE Response SLA:**
- CRITICAL: 24 hours
- HIGH: 7 days
- MEDIUM: 30 days
- LOW: Next release cycle

**Accepted Risks:**
- RSA timing attack (RUSTSEC-2023-0071) - Medium severity, low exploitability in our context
- Review quarterly or when upstream fixes available

---

## Compliance Status

| Requirement | Status |
|-------------|--------|
| No CRITICAL CVEs | ✅ PASS |
| No HIGH CVEs | ✅ PASS |
| MEDIUM CVEs documented | ✅ PASS |
| Mitigation plan exists | ✅ PASS |
| CI integration planned | ⏳ PENDING |

**Production Certification Impact:** Does not block production deployment, but should fix protobuf CVE before launch.
