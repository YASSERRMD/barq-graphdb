# Barq-GraphDB Production Deployment Guide

**Version**: v0.6.0  
**Status**: Production Ready

## 1. Deployment Options

### A. Docker (Recommended for Cloud)

Barq-GraphDB provides a multi-stage Docker image optimized for size and security.

**Prerequisites**: Docker Engine installed.

1. **Build the Image** (or pull from registry):
   ```bash
   docker build -t barq-graphdb:v0.6.0 .
   ```

2. **Run with Docker Compose** (Simplest):
   ```bash
   # docker-compose.yml provided in repo root
   docker-compose up -d
   ```
   This exposes the API on port 3000 and persists data to a Docker volume `barqg-data`.

3. **Run Manually**:
   ```bash
   docker run -d \
     -p 3000:3000 \
     -v barq_data:/data \
     --name barq-server \
     barq-graphdb:v0.6.0
   ```

### B. Binary Deployment (Recommended for On-Prem / Bare Metal)

For maximum performance on dedicated hardware.

1. **Install Dependencies**:
   - Rust toolchain (latest stable) for building.
   - `build-essential` / C++ compiler for certain crate dependencies.

2. **Build Release Binary**:
   ```bash
   cargo build --release --bin barqg_server
   ```
   Binary location: `target/release/barqg_server`

3. **Running**:
   ```bash
   ./target/release/barqg_server \
     --path /var/lib/barq-graphdb \
     --host 0.0.0.0 \
     --port 3000
   ```

4. **Systemd Unit (Example)**:
   Create `/etc/systemd/system/barq-graphdb.service`:
   ```ini
   [Unit]
   Description=Barq-GraphDB API
   After=network.target

   [Service]
   ExecStart=/usr/local/bin/barqg_server --path /var/lib/barq-graphdb
   Restart=always
   User=barq
   LimitNOFILE=65536

   [Install]
   WantedBy=multi-user.target
   ```

---

## 2. Performance Tuning

Based on load testing results (Task 5), Barq-GraphDB scales efficiently with default settings.

- **< 10,000 req/s**: Default settings are sufficient.
- **10,000 - 200,000 req/s**:
  - Ensure file descriptor limits (`ulimit -n`) are raised (e.g., 65,536) to handle concurrent TCP connections.
  - Tokio runtime automatically manages worker threads (defaults to num_cpus).
- **> 200,000 req/s**:
  - Deploy multiple instances behind a load balancer (Nginx / HAProxy).
  - Use a shared storage backend (Phase 6+) or sharding strategy (future).

---

## 3. Monitoring

### Health Check
- **Endpoint**: `GET /health`
- **Expected**: `200 OK`, `{"status":"healthy"...}`
- **Frequency**: Every 10-30 seconds.

### Metrics
- **Endpoint**: `GET /stats`
- **Metics provided**: Node count, Edge count, Vector count, Decision count.
- **Integration**: Poll this endpoint and push to Prometheus/Grafana.

### System Monitoring
- **CPU**: Monitor for high utilization. If >80% consistently, scale up (vertical) or out (horizontal).
- **Memory**: Monitor resident set size (RSS). Memory usage roughly correlates with vector count (2.75KB per node).
- **Disk I/O**: WAL append speed is critical for write throughput. Use SSD/NVMe.

---

## 4. Troubleshooting

### High Latency
1. **Check CPU**: Is serialization blocking?
2. **Check Clients**: Are clients using Keep-Alive? Opening excessive new connections adds TCP handshake latency.
3. **Check Storage**: Is the disk I/O saturated?

### Connection Timeouts
1. **Check File Descriptors**: `ulimit -n`.
2. **Check Backlog**: Kernel TCP backlog settings.

### Database Locking
Barq uses a single writer lock. Heavy write loads may block reads temporarily.
- **Solution**: Batched writes are faster than many small writes. Use efficient write patterns.

---

## 5. Backup & Recovery

**Backup**:
1. Stop the server (recommended for consistency) or flush WAL.
2. Copy the `wal.log` and `version` files from the data directory.
   ```bash
   cp -r /var/lib/barq-graphdb /backup/location
   ```

**Recovery**:
1. Restore the data directory.
2. Start the server. It will replay the WAL on startup.
