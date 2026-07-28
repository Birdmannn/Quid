# Design.md — Create a Quest Flow (Wizard)
 
**Product area:** Creator app · Quests
**Branch:** `design/create-quest-flow`
**Depends on:** `[DESIGN] Creator Quests hub` (wizard entry from hub)
**Status:** Ready for review / closes issue "Create a Quest Flow"
 
---
 
## 1. Overview
 
The Create a Quest flow is a guided, 6-step wizard for founders to define and publish a participation campaign. It launches as a large right-side panel/overlay on top of a dimmed Quests hub, with autosave, draft support, preview, and a publish confirmation. This is a multi-step creator — not a single long-form.
 
---
 
## 2. Entry & Shell
 
### 2.1 Entry Point
- Triggered from "+ Add new Quest" on the Quests hub
- Quests hub remains visible but dimmed/scrim behind the panel (establishes overlay hierarchy, not full navigation away)
### 2.2 Wizard Shell (persistent across all steps)
- **Top bar:**
  - Left: "Exit creator" (text/icon button — triggers confirm-exit if unsaved changes exist, otherwise closes immediately since autosave covers drafts)
  - Right: "Saved" indicator — checkmark + label appears after autosave fires; transitions from "Saving…" → "Saved" with timestamp on hover
- **Title:** "Create a quest"
- **Stepper (6 steps):**
  1. Basics
  2. Eligibility
  3. Tasks
  4. Rewards
  5. Schedule
  6. Review & publish
  - **Completed step:** checkmark icon, muted purple fill
  - **Current step:** purple filled circle/number, bold label
  - **Upcoming step:** muted/outline circle, muted label
  - Steps are tap-navigable once visited (not strictly linear once a step has been completed once)
- **Footer (all steps):**
  - Left: "Back" (hidden on Step 1)
  - Center/utility: "Preview" · "Save as draft"
  - Right: "Continue" (primary, purple; disabled/greyed until required fields on that step are valid)
  - On Step 6 only: primary button becomes **"Publish quest"**
---
 
## 3. Step Specifications
 
### 3.1 Step 1 — Basics
- Quest type cards (selectable, radio-style):
  - **Product testing** (selected/default state shown)
  - **Community participation**
  - Selected card: purple border/fill highlight; unselected: neutral outline
- Title field — character counter "0/90"
- Description field (multi-line textarea)
- Est. completion duration (dropdown/stepper input)
- Participant limit (numeric input)
- Visibility (dropdown: e.g. Public / Unlisted)
- **Continue** enabled once Title + Description + quest type are filled
### 3.2 Step 2 — Eligibility
- "Who can participate" (selector — e.g. Everyone / Specific criteria)
- Max submissions per participant (numeric stepper)
- Eligible countries/regions (multi-select chips/dropdown)
- "Previous winners can participate" — checkbox
- **Advanced eligibility** — expandable/accordion section
  - Additional requirement (free-text textarea) revealed on expand
- **Continue** enabled once core eligibility fields are set (advanced section optional)
### 3.3 Step 3 — Tasks
- Helper copy at top: guidance on writing clear questions and requesting useful evidence
- Reorderable task block list (drag handle per block):
  - Task type selector (Paragraph / Upload / — extensible dropdown)
  - Task title (input)
  - Participant instructions (textarea)
  - "Required" toggle
  - Delete icon (per block)
- "+ Add task block" (secondary button, appends new block)
- **Continue** enabled once at least one valid task block exists
### 3.4 Step 4 — Rewards
- Reward method (dropdown, e.g. "Selected winners")
- Total reward budget (numeric input, XLM)
- Number of winners (numeric input)
- Reward per winner (auto-calculated from budget ÷ winners, or independently editable depending on method — shown as computed field)
- **Summary block:**
  - Total required balance
  - Available wallet balance
  - Remaining balance (turns warning/red state if required > available)
- **Fee line:** Platform fee + network fee estimate, labeled "Estimates — final fees calculated at distribution"
- **Continue** disabled if required balance exceeds available balance
### 3.5 Step 5 — Schedule
- Publish timing toggle: **Publish immediately** vs **Schedule for later**
  - If scheduled: reveals Start date/time field
- Closing (date/time)
- Time zone (dropdown)
- Winner announcement (date/time)
- "Close automatically when participant limit is reached" — checkbox
- **Continue** enabled once required dates are set and logically valid (e.g. Closing > Start)
### 3.6 Step 6 — Review & Publish
- Summary cards (read-only, each with an "Edit" affordance that jumps back to the relevant step):
  - Details (from Basics)
  - Eligibility
  - Rewards
  - Tasks (numbered list)
- Est. completion callout (highlighted info card)
- Primary action: **"Publish quest"**
---
 
## 4. Success State
 
### 4.1 Success Modal
- Rendered centered, over the (now closed/closing) wizard, with the Quests hub visible behind
- Heading: "Quest Published Successfully"
- Supporting copy: e.g. confirmation that the quest is now live and participants can begin submitting
- Actions:
  - "Close" (secondary — returns to Quests hub, Active tab)
  - "View quest" (primary — navigates directly to the published quest's detail/review view)
---
 
## 5. Responsive Behavior
 
### 5.1 Desktop
- Right-side panel overlay, fixed width (e.g. ~640–720px), full viewport height, hub dimmed behind
- Stepper displayed horizontally across the top of the panel (kept consistent with footer-anchored actions)
### 5.2 Tablet
- Panel widens proportionally to occupy more of the viewport (e.g. ~80–90% width) while retaining the dimmed-hub overlay relationship
- Stepper remains horizontal; step labels may collapse to numbers-only with the active step's label shown
### 5.3 Mobile
- Wizard becomes **full-screen** (overlay pattern is not viable at mobile widths)
- Top bar: "Exit creator" (icon) · step indicator (e.g. "Step 2 of 6") replaces full stepper · "Saved" indicator condensed to icon-only
- Full stepper detail (all 6 labeled steps) accessible via a tap-to-expand step list if needed
- Footer actions stack: "Continue"/"Publish quest" full-width primary; "Back," "Preview," "Save as draft" collapse into a secondary row or overflow menu to preserve space
- Success modal becomes full-screen takeover rather than centered modal

---
 
## 10. Submission
 
- **Branch:** `design/create-quest-flow`
- **Depends on:** `[DESIGN] Creator Quests hub` (wizard entry from hub)
- **PR:** `docs(design): Create a quest wizard`