# Monitoring

Set up observability for Needle in production.

## Prometheus Metrics

Needle exports Prometheus metrics at `/metrics`:

```bash
curl http://localhost:3000/metrics
```

### Key Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `needle_tunnels_active` | Gauge | Number of active tunnels |
| `needle_http_requests_total` | Counter | Total HTTP requests |
| `needle_http_request_duration_seconds` | Histogram | Request latency |
| `needle_auth_failures_total` | Counter | Failed auth attempts |
| `needle_errors_total` | Counter | Error count by type |

### Prometheus Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'needle'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
    scrape_interval: 30s
```

Start Prometheus:

```bash
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v /path/to/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus
```

## Grafana Dashboards

Import the Needle dashboard from `dashboards/needle.json` (if available).

Or create custom panels:

### Active Tunnels
```promql
needle_tunnels_active
```

### Request Rate
```promql
rate(needle_http_requests_total[5m])
```

### Error Rate
```promql
rate(needle_errors_total[5m])
```

### Latency (95th percentile)
```promql
histogram_quantile(0.95, rate(needle_http_request_duration_seconds_bucket[5m]))
```

## Alerting

### Prometheus Alerts

Create `alerts.yml`:

```yaml
groups:
  - name: needle
    rules:
      - alert: HighErrorRate
        expr: rate(needle_errors_total[5m]) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"

      - alert: TunnelCapacityHigh
        expr: needle_tunnels_active > 900
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Approaching tunnel capacity ({{ $value }}/1000)"

      - alert: ServiceDown
        expr: up{job="needle"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Needle service is down"
```

## Logging

View logs:

```bash
# SystemD logs
sudo journalctl -u needle -f

# Last 100 lines
sudo journalctl -u needle -n 100

# Errors only
sudo journalctl -u needle -p err
```

### Log Aggregation

For production, use:
- **Loki** - Log aggregation
- **Grafana** - Log visualization
- **Vector** - Log shipping

Example Vector config:

```toml
[sources.needle_logs]
type = "journald"
units = ["needle"]

[sinks.loki]
type = "loki"
inputs = ["needle_logs"]
endpoint = "http://localhost:3100"
```

## Health Checks

### Uptime Monitoring

Use external service to monitor `/health`:

```bash
curl https://api.yourdomain.com/health
```

Expected: `{"status":"ok",...}`

Services:
- **UptimeRobot** - Free tier available
- **Pingdom** - Advanced monitoring
- **StatusCake** - Multi-location checks

### Internal Health Check

Add to cron:

```bash
*/5 * * * * curl -f http://localhost:3000/health || systemctl restart needle
```

## Next Steps

- [Security](./security.md) - Protect your metrics endpoint
- [Troubleshooting](./troubleshooting.md) - Diagnose issues using metrics
