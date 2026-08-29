# E2E Edge Paths: Pause / Cancel / Reject / Capacity Errors

> Issue #344 – Manual + scripted QA for each error code path.
> Depends on: E2E happy path (#343) being verified first.

---

## Error Code Reference

| QuidError | Code | UI Title | Scenario |
|-----------|------|----------|----------|
| MissionNotFound | 1 | Quest not found | Access deleted/invalid mission |
| MissionClosed | 2 | Quest is closed | Submit after cancel/complete |
| MissionFull | 3 | Quest is full | Submit when at capacity |
| AlreadySubmitted | 4 | Already submitted | Hunter submits twice |
| InsufficientFunds | 5 | Insufficient funds | Wallet has < 1 XLM + fees |
| NotAuthorized | 6 | Not authorized | Non-owner tries to approve |
| InvalidState | 8 | Invalid quest state | Generic state machine violation |
| AlreadyPaid | 9 | Already paid | Approve after payout |
| MissionNotOpen | 10 | Quest is paused | Submit while mission is PAUSED |
| SubmissionNotFound | 11 | Submission not found | Approve with wrong ID |
| NotPending | 12 | Submission already reviewed | Double approve/reject |

---

## Path 1 – Pause Blocks Submit (QuidError::MissionNotOpen #10)

**Setup**: Create a mission, pause it via the contract.

- [ ] Open **Hunter Dashboard → All Quest** while mission is PAUSED.
- [ ] Click **Submit Feedback** on the paused quest.
- [ ] **Expected**: UI shows `"Quest is paused – Submissions are temporarily blocked."` banner.
- [ ] The Submit Feedback button should be disabled or removed for paused quests.
- [ ] The on-chain transaction should NOT be invoked.

**API verification:**
```bash
# Mission status should be PAUSED
curl http://localhost:3001/missions/$MISSION_ID | jq '.status'
# Expected: "PAUSED"
```

---

## Path 2 – Cancel Refunds Remaining (QuidError::MissionClosed #2)

**Setup**: Create a mission with 2 hunters already submitted. Cancel the mission.

- [ ] Attempt to submit feedback after mission is CANCELLED.
- [ ] **Expected**: UI shows `"Quest is closed – paused or closed by the founder."` banner.
- [ ] Verify both existing hunters still see their submission in **My Quest** tab.
- [ ] Verify the remaining escrow balance has been refunded to the founder's wallet.

**Horizon balance check:**
```bash
curl "https://horizon-testnet.stellar.org/accounts/$FOUNDER" | jq '.balances'
# XLM should have increased by the unclaimed reward portion
```

---

## Path 3 – Double Submit (QuidError::AlreadySubmitted #4)

**Setup**: Hunter A submits for mission X successfully.

- [ ] Hunter A attempts to submit again for the same mission.
- [ ] **Expected**: UI shows `"Already submitted – Each hunter may submit once."` banner.
- [ ] The **Submit Feedback** button should be replaced by "Submission Recorded" after the first successful submission.
- [ ] The Pending badge (issue #349) should appear and clear after the indexer reflects the first submission.

---

## Path 4 – Capacity Error (QuidError::MissionFull #3)

**Setup**: Create a mission with `maxParticipants: 2`. Two hunters submit.

- [ ] A third hunter attempts to submit.
- [ ] **Expected**: UI shows `"Quest is full – Try a different one."` banner.
- [ ] Quest should be filtered out from **All Quest** / **For You** tabs when full (nice-to-have).

---

## Path 5 – Reject: Human Error Message (QuidError::NotPending #12)

**Setup**: Founder approves a submission.

- [ ] Founder attempts to approve the same submission a second time.
- [ ] **Expected**: UI shows `"Submission already reviewed – Cannot be changed."` banner (409 Conflict).
- [ ] No duplicate payout transaction is invoked.

---

## Path 6 – Reject with Reason

**Setup**: Hunter submits; Founder rejects with a reason.

- [ ] Founder opens the submission detail and clicks **Reject**.
- [ ] Enters reason: `"The feedback was too brief and off-topic."`
- [ ] Clicks **Confirm Reject**.
- [ ] **Expected**: Submission status changes to `Rejected` in the UI.
- [ ] Hunter can see the rejection reason in **My Quest → Submission detail**.

**API verification:**
```bash
curl -H "Authorization: Bearer $FOUNDER_TOKEN" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"reason": "Too brief"}' \
  http://localhost:3001/missions/$MISSION_ID/submissions/$SUB_ID/reject \
  | jq '{status: .status, reason: .rejectionReason}'
# Expected: { "status": "REJECTED", "reason": "Too brief" }
```

---

## Path 7 – Non-owner Authorization Error (QuidError::NotAuthorized #6)

**Setup**: Hunter A tries to approve Hunter B's submission.

- [ ] Hunter A calls the approve endpoint directly (or via UI).
- [ ] **Expected**: 403 Forbidden response from the API.
- [ ] UI must not display the **Approve / Reject** buttons to non-owners.

---

## Automation Script

See [`scripts/e2e-edge-paths.sh`](../../scripts/e2e-edge-paths.sh) for a
curl-based script that exercises all API-level error paths.

---

## Acceptance Criteria Status

| Criterion | How verified |
|-----------|-------------|
| Pause blocks submit | Path 1 above + unit test `missions.edge-paths.spec.ts` |
| Cancel refunds remaining | Path 2 + Horizon balance check |
| Double submit shows human error | Path 3 + unit test |
| Double approve shows human error | Path 5 + unit test |
| Non-owner gets 403 | Path 7 + unit test |
| Rejection reason persisted | Path 6 + unit test |
