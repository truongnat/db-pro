#!/usr/bin/env bash
# Regenerate gitignored memory mirrors and pipeline JSON (VERIFICATION §9).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
PY="${PYTHON:-python3}"
SCRIPTS=".agents/scripts"

echo "==> build inventory"
"$PY" "$SCRIPTS/build-knowledge-inventory.py" >/dev/null

echo "==> migrate pending (pass 1)"
"$PY" "$SCRIPTS/migrate-pending-knowledge.py" >/dev/null

echo "==> reconcile v2 (pass 1)"
REC1="$("$PY" "$SCRIPTS/reconcile-knowledge-sources-v2.py")"
echo "$REC1"

echo "==> classify groups"
"$PY" "$SCRIPTS/classify-knowledge-groups.py" >/dev/null

echo "==> migrate pending (post-classify)"
"$PY" "$SCRIPTS/migrate-pending-knowledge.py" >/dev/null

echo "==> reconcile v2 (pass 2)"
REC2="$("$PY" "$SCRIPTS/reconcile-knowledge-sources-v2.py")"
echo "$REC2"

SCOPE="$(echo "$REC2" | "$PY" -c "import sys,json; print(json.load(sys.stdin).get('scope_complete'))")"
if [[ "$SCOPE" != "True" && "$SCOPE" != "true" ]]; then
  echo "FAIL: scope_complete=$SCOPE" >&2
  exit 1
fi

echo "==> mirror-utils + skills"
"$PY" "$SCRIPTS/test-knowledge-mirror-utils.py"
"$SCRIPTS/validate-skills.sh"

echo "OK: agents memory bootstrap (see .agents/memory/KNOWLEDGE_RECONCILIATION_V2.json)"
