#!/usr/bin/env bash
# provision-test-fixtures.sh — idempotent bootstrap for the provider integration
# fixtures the `#[ignore]`d suites need on a developer machine (issue #441 wave).
#
# Covers:
#   1. SQL Server 2022 container   -> sqlserver_integration  (SQLSERVER_URL)
#   2. repo docker-compose MySQL   -> mysql_integration      (DATABASE_URL + MYSQL_REFERENCE_DATABASE)
#   3. SSH fixture (openssh server + loopback PG proxy)      -> ssh_backup_runtime_verification (DB_PRO_SSH_*)
#
# SAFETY / SECRETS: every credential below is a TEST-ONLY fixture value, either
# generated here or already committed in docker-compose.yml. Never reuse these
# passwords outside a disposable local fixture; never point these suites at real
# data. The script leaves containers running (`--restart unless-stopped`).
#
# Notes:
#   - `mcr.microsoft.com/mssql/server:2022-latest` publishes amd64 only; on Apple
#     Silicon it runs under Rosetta/qemu (`--platform linux/amd64`).
#   - The Homebrew MySQL on :3306 is intentionally NOT touched: its admin
#     credentials are not discoverable in-repo, so `MYSQL_REFERENCE_DATABASE`
#     provisioning uses the repo-owned compose MySQL 8.4 fixture on :33306
#     instead (root password `dbpro_test`, already in docker-compose.yml).
#   - Prereqs: docker daemon running, `mysql`/`mysqladmin` client on PATH for
#     step 2, `ssh`/`ssh-keyscan` and `pg_dump`/`psql` (libpq >= server major)
#     on PATH for step 3, live PostgreSQL on localhost:5432 with
#     dbpro/dbpro_test and dbpro_fixture (see docker-compose.yml).
set -euo pipefail

# --- test-only credentials (see SAFETY note above) -------------------------
MSSQL_SA_PASSWORD='DbProFixture2026!'          # percent-encoded in the URL: ! -> %21
SQLSERVER_URL="sqlserver://sa:DbProFixture2026%21@127.0.0.1:1433/dbpro_fixture"
MYSQL_ROOT_PASSWORD='dbpro_test'               # from docker-compose.yml (fixture-only)
MYSQL_FIXTURE_DSN="mysql://root:${MYSQL_ROOT_PASSWORD}@127.0.0.1:33306/dbpro_fixture"
SSH_KEY_DIR="${HOME}/.dbpro-ssh-fixture"
SSH_USER='linuxserver.io'                      # default user of lscr.io/linuxserver/openssh-server

log() { printf '==> %s\n' "$*"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

docker info >/dev/null 2>&1 || die "docker daemon is not running"

# ---------------------------------------------------------------------------
# 1. SQL Server 2022 container (dbpro-mssql-fixture)
# ---------------------------------------------------------------------------
MSSQL_CONTAINER=dbpro-mssql-fixture
if docker ps -a --format '{{.Names}}' | grep -qx "$MSSQL_CONTAINER"; then
  docker ps --format '{{.Names}}' | grep -qx "$MSSQL_CONTAINER" \
    || docker start "$MSSQL_CONTAINER" >/dev/null
  log "SQL Server container reused: $MSSQL_CONTAINER"
else
  if nc -z -w1 localhost 1433; then die "port 1433 already in use by something else"; fi
  log "Creating SQL Server container (amd64 image; emulated on Apple Silicon)"
  docker run -d --name "$MSSQL_CONTAINER" --platform linux/amd64 \
    --restart unless-stopped \
    -e ACCEPT_EULA=Y -e MSSQL_PID=developer \
    -e "MSSQL_SA_PASSWORD=${MSSQL_SA_PASSWORD}" \
    -p 1433:1433 \
    -v dbpro_mssql_data:/var/opt/mssql \
    mcr.microsoft.com/mssql/server:2022-latest >/dev/null
fi

# Wait for readiness: first look for the startup marker, then poll with sqlcmd.
log "Waiting for SQL Server to accept connections"
for _ in $(seq 1 60); do
  if docker exec "$MSSQL_CONTAINER" /opt/mssql-tools18/bin/sqlcmd -C -S localhost \
       -U sa -P "$MSSQL_SA_PASSWORD" -Q "SELECT 1" >/dev/null 2>&1; then
    break
  fi
  docker ps --format '{{.Names}}' | grep -qx "$MSSQL_CONTAINER" \
    || { docker logs --tail 40 "$MSSQL_CONTAINER"; die "SQL Server container exited — see logs above"; }
  sleep 5
done
docker exec "$MSSQL_CONTAINER" /opt/mssql-tools18/bin/sqlcmd -C -S localhost \
  -U sa -P "$MSSQL_SA_PASSWORD" -Q "SELECT 1" >/dev/null \
  || die "SQL Server not ready after 300s"

docker exec "$MSSQL_CONTAINER" /opt/mssql-tools18/bin/sqlcmd -C -S localhost \
  -U sa -P "$MSSQL_SA_PASSWORD" \
  -Q "IF DB_ID('dbpro_fixture') IS NULL CREATE DATABASE dbpro_fixture;" >/dev/null
log "SQL Server fixture database dbpro_fixture ensured"

# ---------------------------------------------------------------------------
# 2. MySQL fixture (repo docker-compose service on :33306) + reference database
# ---------------------------------------------------------------------------
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v mysql >/dev/null || die "mysql client not on PATH (needed for step 2)"
log "Starting repo compose MySQL fixture (docker-compose.yml service: mysql)"
(cd "$REPO_ROOT" && docker compose up -d mysql >/dev/null)
for _ in $(seq 1 30); do
  if mysqladmin -h 127.0.0.1 -P 33306 -u root -p"$MYSQL_ROOT_PASSWORD" ping 2>/dev/null | grep -q alive; then break; fi
  sleep 2
done
mysqladmin -h 127.0.0.1 -P 33306 -u root -p"$MYSQL_ROOT_PASSWORD" ping 2>/dev/null | grep -q alive \
  || die "compose MySQL did not become healthy on 127.0.0.1:33306"
# The cross-database FK test creates same-named tables in BOTH databases with the
# single DATABASE_URL connection, so one server must allow both via one account.
# root@compose fixture has global privileges; the reference DB only needs to exist.
mysql -h 127.0.0.1 -P 33306 -u root -p"$MYSQL_ROOT_PASSWORD" \
  -e "CREATE DATABASE IF NOT EXISTS dbpro_reference;" 2>/dev/null
log "MySQL fixture databases ensured: dbpro_fixture + dbpro_reference (:33306)"

# ---------------------------------------------------------------------------
# 3. SSH fixture: openssh server container + socat loopback proxy to host PG
# ---------------------------------------------------------------------------
# Mirrors the CI topology (.github/workflows/ci.yml "Configure live SSH backup
# fixture"): the SSH server must reach the target PostgreSQL on ITS OWN
# 127.0.0.1:5432, because the suites dial DB_PRO_SSH_TARGET_* both directly
# (loopback on the host) and as the ssh -L remote destination (loopback inside
# the sshd container). The socat sidecar shares the sshd container's network
# namespace and forwards 127.0.0.1:5432 -> host.docker.internal:5432.
SSH_CONTAINER=dbpro-sshd-fixture
SSH_PROXY_CONTAINER=dbpro-ssh-pg-proxy
SSH_PORT=2222
command -v ssh-keyscan >/dev/null || die "ssh-keyscan not on PATH (needed for step 3)"

if [ ! -f "$SSH_KEY_DIR/id_ed25519" ]; then
  log "Generating TEST-ONLY SSH keypair in $SSH_KEY_DIR"
  mkdir -p "$SSH_KEY_DIR" && chmod 700 "$SSH_KEY_DIR"
  ssh-keygen -q -t ed25519 -N '' -C 'dbpro-ssh-test-fixture' -f "$SSH_KEY_DIR/id_ed25519"
fi

SSHD_RECREATED=0
if docker ps --format '{{.Names}}' | grep -qx "$SSH_CONTAINER"; then
  log "SSH server container reused: $SSH_CONTAINER"
else
  if docker ps -a --format '{{.Names}}' | grep -qx "$SSH_CONTAINER"; then
    docker start "$SSH_CONTAINER" >/dev/null
  else
    if nc -z -w1 localhost "$SSH_PORT"; then die "port $SSH_PORT already in use"; fi
    log "Creating SSH server container (lscr.io/linuxserver/openssh-server)"
    docker run -d --name "$SSH_CONTAINER" --restart unless-stopped \
      -p 127.0.0.1:${SSH_PORT}:2222 \
      -e PUID=1000 -e PGID=1000 \
      -e SSH_PORT=2222 -e PUBLIC_KEY_FILE=/keys/id_ed25519.pub \
      -v "$SSH_KEY_DIR:/keys:ro" \
      lscr.io/linuxserver/openssh-server:latest >/dev/null
    SSHD_RECREATED=1
  fi
  log "Waiting for sshd to come up"
  for _ in $(seq 1 30); do
    if nc -z -w1 localhost "$SSH_PORT"; then break; fi
    sleep 1
  done
  # The image hardens sshd with `AllowTcpForwarding no` and ignores OPTIONAL_ARGS
  # on current tags; patch the generated config and let s6 respawn the master.
  # (Fresh container only: a recreated master re-reads the patched file on boot
  # within this same run, so reuse paths never need the patch.)
  docker exec "$SSH_CONTAINER" sh -c '
    command grep -q "^AllowTcpForwarding no" /config/sshd/sshd_config &&
    sed -i "s/^AllowTcpForwarding no/AllowTcpForwarding yes/" /config/sshd/sshd_config &&
    pkill -f "sshd.pam.*listener" || true'
  sleep 3
fi

if [ "$SSHD_RECREATED" -eq 1 ] || ! docker ps --format '{{.Names}}' | grep -qx "$SSH_PROXY_CONTAINER"; then
  if [ "$SSHD_RECREATED" -eq 1 ]; then docker rm -f "$SSH_PROXY_CONTAINER" >/dev/null 2>&1 || true; fi
  log "Creating socat loopback proxy (sshd-netns 127.0.0.1:5432 -> host PostgreSQL)"
  # TCP4: pins the proxy to IPv4 — host.docker.internal also has an IPv6 F-DNS
  # record in Docker Desktop that breaks the PostgreSQL handshake.
  docker run -d --name "$SSH_PROXY_CONTAINER" --restart unless-stopped \
    --network "container:$SSH_CONTAINER" \
    alpine/socat "TCP-LISTEN:5432,fork,reuseaddr" "TCP4:host.docker.internal:5432" >/dev/null
  sleep 2
fi

# sshd host key must be trusted: the product tunnel never disables host key
# verification (crates/infrastructure/src/ssh/tunnel.rs), and the suite
# deliberately inherits the OpenSSH known-hosts policy. Entries are tagged so
# this script can replace its own previous lines idempotently.
log "Refreshing known_hosts entry for the fixture sshd (marker: # dbpro-ssh-fixture)"
touch "$HOME/.ssh/known_hosts"
grep -v 'dbpro-ssh-fixture' "$HOME/.ssh/known_hosts" > "$HOME/.ssh/known_hosts.tmp" \
  && mv "$HOME/.ssh/known_hosts.tmp" "$HOME/.ssh/known_hosts" || true
ssh-keyscan -p "$SSH_PORT" 127.0.0.1 2>/dev/null | sed 's/$/ # dbpro-ssh-fixture/' >> "$HOME/.ssh/known_hosts"

ssh -i "$SSH_KEY_DIR/id_ed25519" -p "$SSH_PORT" -o BatchMode=yes \
  "${SSH_USER}@127.0.0.1" 'nc -z -w3 127.0.0.1 5432 && echo TUNNEL_CHAIN_OK' >/dev/null \
  || die "SSH tunnel chain smoke test failed (sshd forward or socat proxy down, or PostgreSQL not on host :5432)"
log "SSH tunnel chain verified (key auth + forwarding + loopback PG proxy)"

# ---------------------------------------------------------------------------
# Reproduce: exported env for each suite
# ---------------------------------------------------------------------------
cat <<EOF

Fixture containers are left RUNNING (--restart unless-stopped).

# 1) SQL Server — sqlserver_integration (currently 2 PASS / 3 FAIL, see issue #441 findings)
export SQLSERVER_URL='${SQLSERVER_URL}'
cargo test -p db-pro-infrastructure --test sqlserver_integration -- --ignored --nocapture

# 2) MySQL — mysql_integration incl. cross-database FK test (12/12 PASS on the compose fixture;
#    the Homebrew :3306 instance is admin-BLOCKED, do not use it for MYSQL_REFERENCE_DATABASE)
export DATABASE_URL='${MYSQL_FIXTURE_DSN}'
export MYSQL_REFERENCE_DATABASE='dbpro_reference'
cargo test -p db-pro-infrastructure --test mysql_integration -- --ignored --nocapture

# 3) SSH — ssh_backup_runtime_verification (1/1 PASS; requires live PostgreSQL 18 client tools)
export DB_PRO_SSH_HOST='127.0.0.1'
export DB_PRO_SSH_PORT='${SSH_PORT}'
export DB_PRO_SSH_USER='${SSH_USER}'
export DB_PRO_SSH_KEY='${SSH_KEY_DIR}/id_ed25519'
export DB_PRO_SSH_TARGET_HOST='127.0.0.1'
export DB_PRO_SSH_TARGET_PORT='5432'
export DB_PRO_SSH_DATABASE='dbpro_fixture'
export DB_PRO_SSH_USERNAME='dbpro'
export DB_PRO_SSH_PASSWORD='dbpro_test'
cargo test -p db-pro-infrastructure --test ssh_backup_runtime_verification -- --ignored --nocapture

# After a machine reboot the containers auto-start; or:
docker start dbpro-mssql-fixture db-pro-mysql-1 ${SSH_CONTAINER} ${SSH_PROXY_CONTAINER}
EOF
