#!/usr/bin/env bash
set -euo pipefail

# Starts a real local Engine + PostgreSQL stack against a deterministic RPC fixture.
# --serve keeps the stack alive for the Console cross-repository smoke check.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="${QS_DEMO_STATE_DIR:-$(mktemp -d)}"
API_PORT="${QS_DEMO_API_PORT:-18080}"
RPC_PORT="${QS_DEMO_RPC_PORT:-18081}"
DB_PORT="${QS_DEMO_DB_PORT:-55432}"
SERVE_ONLY=false

if [[ "${1:-}" == "--serve" ]]; then
  SERVE_ONLY=true
fi
mkdir -p "$STATE_DIR"

CONTAINER="quorumscope-demo-${$}"
DB_URL="postgres://quorumscope:quorumscope_demo@127.0.0.1:${DB_PORT}/quorumscope"
API_URL="http://127.0.0.1:${API_PORT}"
RPC_URL="http://127.0.0.1:${RPC_PORT}"

cleanup() {
  set +e
  [[ -n "${API_PID:-}" ]] && kill "$API_PID" 2>/dev/null
  [[ -n "${RPC_PID:-}" ]] && kill "$RPC_PID" 2>/dev/null
  docker rm -f "$CONTAINER" >/dev/null 2>&1
}
trap cleanup EXIT INT TERM

command -v docker >/dev/null || { echo "docker is required for the deterministic fixture demo" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 is required for the deterministic fixture demo" >&2; exit 1; }

docker run --rm -d --name "$CONTAINER" \
  -e POSTGRES_USER=quorumscope \
  -e POSTGRES_PASSWORD=quorumscope_demo \
  -e POSTGRES_DB=quorumscope \
  -p "127.0.0.1:${DB_PORT}:5432" \
  postgres:16-alpine >/dev/null
until docker exec "$CONTAINER" pg_isready -U quorumscope -d quorumscope >/dev/null 2>&1; do
  sleep 1
done

cargo run --quiet -p quorumscope-api --example demo_fixture >"$STATE_DIR/fixture.json"
python3 "$ROOT_DIR/scripts/demo-rpc.py" --fixture "$STATE_DIR/fixture.json" --port "$RPC_PORT" &
RPC_PID=$!

DATABASE_URL="$DB_URL" cargo run --quiet -p quorumscope-cli -- init
DATABASE_URL="$DB_URL" STELLAR_RPC_URL="$RPC_URL" NETWORK_NAME=deterministic-demo \
  NETWORK_PASSPHRASE='Test SDF Network ; September 2015' \
  cargo run --quiet -p quorumscope-cli -- index once

DATABASE_URL="$DB_URL" API_BIND="127.0.0.1:${API_PORT}" STELLAR_RPC_URL="$RPC_URL" \
  VERIFIED_PROTOCOL_MAX=29 NETWORK_NAME=deterministic-demo \
  ALLOWED_ORIGINS="http://127.0.0.1:3100" \
  cargo run --quiet -p quorumscope-cli -- serve &
API_PID=$!

until curl --silent --fail "$API_URL/health/ready" >/dev/null; do
  sleep 1
done

cat >"$STATE_DIR/demo.env" <<EOF
QS_DEMO_API_URL=$API_URL
QS_DEMO_FIXTURE=$STATE_DIR/fixture.json
QS_DEMO_BLOCKED_XDR=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["blocked_transaction_xdr"])' "$STATE_DIR/fixture.json")
QS_DEMO_BYPASSED_XDR=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["bypassed_transaction_xdr"])' "$STATE_DIR/fixture.json")
EOF

if [[ "$SERVE_ONLY" == false ]]; then
  echo "Deterministic fixture demo: $API_URL"
  curl --silent --fail "$API_URL/api/v1/network"
  curl --silent --fail "$API_URL/api/v1/freeze-state"
  curl --silent --fail "$API_URL/api/v1/frozen-keys"
  curl --silent --fail "$API_URL/api/v1/bypasses"
  curl --silent --fail "$API_URL/api/v1/impact"
  BLOCKED_XDR="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["blocked_transaction_xdr"])' "$STATE_DIR/fixture.json")"
  BYPASSED_XDR="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["bypassed_transaction_xdr"])' "$STATE_DIR/fixture.json")"
  curl --silent --fail -H 'content-type: application/json' -d "{\"transaction_xdr\":\"$BLOCKED_XDR\"}" "$API_URL/api/v1/preflight"
  curl --silent --fail -H 'content-type: application/json' -d "{\"transaction_xdr\":\"$BYPASSED_XDR\"}" "$API_URL/api/v1/preflight"
  echo
  echo "The output above is deterministic fixture data, not live Testnet state."
else
  echo "Deterministic fixture API is ready at $API_URL"
  echo "Source environment: $STATE_DIR/demo.env"
  wait "$API_PID"
fi
