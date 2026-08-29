#!/usr/bin/env bash
# =============================================================================
# scripts/e2e-happy-path.sh
# Issue #343 – E2E happy path: Founder → Hunter → Payout (curl smoke test)
#
# Usage:
#   API_URL=http://localhost:3001 \
#   FOUNDER_JWT=<token> \
#   HUNTER_JWT=<token> \
#   HUNTER_ADDRESS=G... \
#   bash scripts/e2e-happy-path.sh
#
# The script uses pre-existing JWT tokens (obtained from the /auth/verify
# endpoint during manual SEP-10 sign-in).  It does NOT perform wallet
# signatures – those must be done via the Freighter UI or a separate
# signing script.
# =============================================================================

set -euo pipefail

API_URL="${API_URL:-http://localhost:3001}"
FOUNDER_JWT="${FOUNDER_JWT:?FOUNDER_JWT env var required}"
HUNTER_JWT="${HUNTER_JWT:?HUNTER_JWT env var required}"
HUNTER_ADDRESS="${HUNTER_ADDRESS:?HUNTER_ADDRESS env var required}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}✓ PASS${NC}  $1"; }
fail() { echo -e "${RED}✗ FAIL${NC}  $1"; exit 1; }
info() { echo -e "${YELLOW}»${NC} $1"; }

# ---------------------------------------------------------------------------
# 1. Health check
# ---------------------------------------------------------------------------
info "Step 0: API health check"
HEALTH=$(curl -sf "${API_URL}/health" | jq -r '.status // "ok"') || fail "API not reachable at ${API_URL}"
pass "API is up (status: ${HEALTH})"

# ---------------------------------------------------------------------------
# 2. Verify founder can list missions
# ---------------------------------------------------------------------------
info "Step 1: Founder lists public missions"
MISSIONS=$(curl -sf -H "Authorization: Bearer ${FOUNDER_JWT}" "${API_URL}/missions")
OPEN_COUNT=$(echo "$MISSIONS" | jq '[.[] | select(.status=="OPEN")] | length')
pass "Found ${OPEN_COUNT} OPEN missions"

# ---------------------------------------------------------------------------
# 3. Grab most recent OPEN mission for testing
# ---------------------------------------------------------------------------
MISSION_ID=$(echo "$MISSIONS" | jq -r '[.[] | select(.status=="OPEN")] | first | .id // empty')
if [[ -z "$MISSION_ID" ]]; then
  fail "No OPEN mission found – create one via the UI before running this script"
fi
pass "Using mission: ${MISSION_ID}"

# ---------------------------------------------------------------------------
# 4. Verify hunter can see the mission
# ---------------------------------------------------------------------------
info "Step 2: Hunter fetches mission detail"
MISSION=$(curl -sf "${API_URL}/missions/${MISSION_ID}")
MISSION_TITLE=$(echo "$MISSION" | jq -r '.title')
pass "Mission title: '${MISSION_TITLE}'"

# ---------------------------------------------------------------------------
# 5. Check for an existing PENDING submission from the hunter
# ---------------------------------------------------------------------------
info "Step 3: Founder checks submissions on the mission"
SUBMISSIONS=$(curl -sf -H "Authorization: Bearer ${FOUNDER_JWT}" \
  "${API_URL}/missions/${MISSION_ID}/submissions")

HUNTER_SUB=$(echo "$SUBMISSIONS" | jq --arg addr "$HUNTER_ADDRESS" \
  '[.[] | select(.hunterAddress == $addr)] | first // empty')

if [[ -z "$HUNTER_SUB" || "$HUNTER_SUB" == "null" ]]; then
  echo -e "${YELLOW}  (no submission from hunter yet – complete Step 2 via UI first)${NC}"
  info "Checklist so far: Founder created mission ✓ | Hunter submit: PENDING UI action"
  exit 0
fi

SUBMISSION_ID=$(echo "$HUNTER_SUB" | jq -r '.id')
SUB_STATUS=$(echo "$HUNTER_SUB" | jq -r '.status')
pass "Found submission ${SUBMISSION_ID} with status ${SUB_STATUS}"

# ---------------------------------------------------------------------------
# 6. Approve the submission if it is still PENDING
# ---------------------------------------------------------------------------
if [[ "$SUB_STATUS" == "PENDING" ]]; then
  info "Step 4: Founder approves submission"
  APPROVE_RESULT=$(curl -sf -X POST \
    -H "Authorization: Bearer ${FOUNDER_JWT}" \
    "${API_URL}/missions/${MISSION_ID}/submissions/${SUBMISSION_ID}/approve")
  NEW_STATUS=$(echo "$APPROVE_RESULT" | jq -r '.status')
  [[ "$NEW_STATUS" == "APPROVED" ]] || fail "Approve did not return APPROVED (got: ${NEW_STATUS})"
  pass "Submission approved successfully (status: ${NEW_STATUS})"
elif [[ "$SUB_STATUS" == "APPROVED" || "$SUB_STATUS" == "PAID" ]]; then
  pass "Submission already in terminal state: ${SUB_STATUS}"
else
  fail "Unexpected submission status: ${SUB_STATUS}"
fi

# ---------------------------------------------------------------------------
# 7. Verify dashboard reflects final state
# ---------------------------------------------------------------------------
info "Step 5: Verify final state via API"
FINAL_SUB=$(curl -sf -H "Authorization: Bearer ${FOUNDER_JWT}" \
  "${API_URL}/missions/${MISSION_ID}/submissions" | \
  jq --arg id "$SUBMISSION_ID" '.[] | select(.id == $id)')
FINAL_STATUS=$(echo "$FINAL_SUB" | jq -r '.status')

[[ "$FINAL_STATUS" == "APPROVED" || "$FINAL_STATUS" == "PAID" ]] || \
  fail "Final status is not APPROVED/PAID (got: ${FINAL_STATUS})"
pass "Dashboard reflects final status: ${FINAL_STATUS}"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  E2E Happy Path: ALL CHECKS PASSED     ${NC}"
echo -e "${GREEN}========================================${NC}"
echo "  Mission ID  : ${MISSION_ID}"
echo "  Submission  : ${SUBMISSION_ID}"
echo "  Final Status: ${FINAL_STATUS}"
echo ""
echo "Next: verify hunter balance via Horizon:"
echo "  curl https://horizon-testnet.stellar.org/accounts/${HUNTER_ADDRESS} | jq '.balances'"
