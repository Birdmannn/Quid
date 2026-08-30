#!/usr/bin/env bash
# =============================================================================
# scripts/e2e-edge-paths.sh
# Issue #344 – QA script for pause/cancel/reject/capacity error code paths
#
# Usage:
#   API_URL=http://localhost:3001 \
#   FOUNDER_JWT=<token> \
#   HUNTER_A_JWT=<token> \
#   MISSION_ID=<uuid> \
#   SUBMISSION_ID=<uuid> \
#   bash scripts/e2e-edge-paths.sh
#
# Pre-conditions:
#   - Mission exists with at least one PENDING submission
#   - Both FOUNDER_JWT and HUNTER_A_JWT are valid
# =============================================================================

set -euo pipefail

API_URL="${API_URL:-http://localhost:3001}"
FOUNDER_JWT="${FOUNDER_JWT:?FOUNDER_JWT env var required}"
HUNTER_A_JWT="${HUNTER_A_JWT:?HUNTER_A_JWT env var required}"
MISSION_ID="${MISSION_ID:?MISSION_ID env var required}"
SUBMISSION_ID="${SUBMISSION_ID:?SUBMISSION_ID env var required}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

pass()    { echo -e "${GREEN}✓ PASS${NC}  $1"; }
fail()    { echo -e "${RED}✗ FAIL${NC}  $1"; exit 1; }
info()    { echo -e "${YELLOW}»${NC} $1"; }
section() { echo -e "\n${BLUE}━━━ $1 ━━━${NC}"; }

# ---------------------------------------------------------------------------
# Path 6: Reject submission with reason
# ---------------------------------------------------------------------------
section "Path 6 – Reject with reason"
info "Rejecting submission ${SUBMISSION_ID} with a reason..."

REJECT_RESULT=$(curl -sf -X POST \
  -H "Authorization: Bearer ${FOUNDER_JWT}" \
  -H "Content-Type: application/json" \
  -d '{"reason": "Feedback was too brief and off-topic."}' \
  "${API_URL}/missions/${MISSION_ID}/submissions/${SUBMISSION_ID}/reject")

REJECT_STATUS=$(echo "$REJECT_RESULT" | jq -r '.status')
REJECT_REASON=$(echo "$REJECT_RESULT" | jq -r '.rejectionReason // empty')

[[ "$REJECT_STATUS" == "REJECTED" ]] || fail "Expected REJECTED, got: ${REJECT_STATUS}"
[[ "$REJECT_REASON" == "Feedback was too brief and off-topic." ]] || \
  fail "Rejection reason mismatch: '${REJECT_REASON}'"
pass "Submission rejected with correct status and reason"

# ---------------------------------------------------------------------------
# Path 5: Double-reject guard (QuidError #12 – NotPending)
# ---------------------------------------------------------------------------
section "Path 5 – Double-reject guard (409 Conflict)"
info "Attempting to reject the same submission again (should return 409)..."

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
  -H "Authorization: Bearer ${FOUNDER_JWT}" \
  -H "Content-Type: application/json" \
  -d '{"reason": "Duplicate reject attempt"}' \
  "${API_URL}/missions/${MISSION_ID}/submissions/${SUBMISSION_ID}/reject")

[[ "$HTTP_CODE" == "409" ]] || fail "Expected 409 Conflict, got: ${HTTP_CODE}"
pass "Double-reject correctly returns 409 Conflict"

# ---------------------------------------------------------------------------
# Path 7: Non-owner authorization (QuidError #6 – NotAuthorized)
# ---------------------------------------------------------------------------
section "Path 7 – Non-owner authorization (403 Forbidden)"
info "Hunter attempts to approve a submission (should return 403)..."

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
  -H "Authorization: Bearer ${HUNTER_A_JWT}" \
  "${API_URL}/missions/${MISSION_ID}/submissions/${SUBMISSION_ID}/approve")

[[ "$HTTP_CODE" == "403" ]] || fail "Expected 403 Forbidden, got: ${HTTP_CODE}"
pass "Non-owner correctly receives 403 Forbidden"

# ---------------------------------------------------------------------------
# Path 11: Submission not found (QuidError #11)
# ---------------------------------------------------------------------------
section "Path 11 – Submission not found (404)"
info "Attempting to approve a nonexistent submission ID..."

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
  -H "Authorization: Bearer ${FOUNDER_JWT}" \
  "${API_URL}/missions/${MISSION_ID}/submissions/nonexistent-submission-id/approve")

[[ "$HTTP_CODE" == "404" ]] || fail "Expected 404 Not Found, got: ${HTTP_CODE}"
pass "Nonexistent submission correctly returns 404"

# ---------------------------------------------------------------------------
# Path 1: Mission not found (QuidError #1)
# ---------------------------------------------------------------------------
section "Path 1 – Mission not found (404)"
info "Fetching a nonexistent mission..."

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  "${API_URL}/missions/nonexistent-mission-uuid-00000000")

[[ "$HTTP_CODE" == "404" ]] || fail "Expected 404 Not Found, got: ${HTTP_CODE}"
pass "Nonexistent mission correctly returns 404"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}  E2E Edge Paths: ALL API CHECKS PASSED        ${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo "Remaining paths require Soroban contract interaction:"
echo "  Path 2 (Cancel refunds)   – use Freighter + monitor Horizon balance"
echo "  Path 3 (Capacity error)   – deploy mission with maxParticipants: 2"
echo "  Path 4 (AlreadySubmitted) – submit twice with same hunter wallet"
echo "  Path 10 (Pause blocks)    – pause mission then attempt submit_feedback"
