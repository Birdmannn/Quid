# Design.md — Creator Quests Hub

**Product area:** Creator app · Quests
**Branch:** `design/creator-quests-hub`
**Status:** Ready for review / closes issue "Creator Quests Hub — Design Spec"

---

## 1. Overview

The Quests hub is the founder-facing home for creating, managing, and reviewing participation campaigns ("quests"). It surfaces three lifecycle states — **Active**, **Drafted**, **Completed** — behind a tabbed interface, with consistent chrome (sidebar, top bar) and a unified quest row/card pattern whose contents and actions adapt per tab.

This document specifies layout, content, states, and components across **desktop, tablet, and mobile**, for both **empty** and **populated** variants.

---

## 2. Theme & Foundations

- **Theme:** Dark Quid theme
- **Primary action color:** Purple (buttons, active tab underline, links)
- **Typography:** Clean sans-serif hierarchy
  - H1 (page title): Bold, largest scale — "Quests"
  - Subtitle: Regular weight, muted/secondary text color
  - Quest title: Semibold, primary text color
  - Meta text: Regular, small scale, muted/secondary text color, paired with icons
- **Surface layers:** Sidebar and top bar sit on a darker base layer; quest rows/cards sit on an elevated dark surface with subtle border or shadow for separation.

---

## 3. Chrome (persistent across all states)

### 3.1 Left Sidebar
- Quid logo + "Creators" label (workspace context)
- Primary nav:
  - Dashboard
  - **Quests** (active — highlighted state, purple indicator)
  - Wallet
- Secondary nav: Help center
- Footer: Profile block — avatar, name, wallet short label (truncated address)

### 3.2 Top Bar
- Center: Search input, placeholder "Search Stellar"
- Right: XLM balance pill (icon + amount), notification bell (with unread indicator dot when applicable)

### 3.3 Responsive Chrome Behavior
- **Desktop:** Sidebar fixed/expanded, top bar full width with centered search
- **Tablet:** Sidebar collapses to icon-only rail (labels on hover/tap), search remains centered but narrower
- **Mobile:** Sidebar becomes a bottom nav bar or slide-out drawer (hamburger trigger); top bar retains balance pill + bell, search collapses to an icon that expands to a full-width field on tap

---

## 4. Page Header

- **Title:** "Quests"
- **Subtitle:** "Create, manage and review your participation campaigns."
- **Primary CTA (top right):** "+ Add new Quest" (purple filled button)

**Responsive behavior:**
- Desktop/Tablet: Title/subtitle left-aligned, CTA top-right, same row
- Mobile: Title/subtitle stack full-width; CTA becomes full-width button below subtitle, or sticky at bottom of viewport

---

## 5. Tabs

- Labels with live counts: `Active Quest (n)` · `Drafted (n)` · `Completed (n)`
- Active tab indicated by purple underline + bolded label
- Inactive tabs: muted text, no underline
- **Mobile:** Tabs become horizontally scrollable (swipeable) pill/segment control if space-constrained

---

## 6. Tab States

### 6.1 Active Quest — Empty State
- Centered illustration/icon (brand-appropriate, muted purple accent)
- Heading: "No Active Quest."
- Helper copy: e.g. "Launch your first campaign to start collecting responses from the community."
- Centered CTA: "+ Add new Quest" (purple filled button)

*(Drafted and Completed follow the same empty-state pattern with tab-appropriate copy, e.g. "No drafts yet." / "No completed quests yet.")*

### 6.2 Active Quest — Populated
Quest rows/cards showing:
- Thumbnail / brand mark (left-aligned, rounded square)
- Title + status pill — e.g. "Reviewing submissions" (purple/amber pill)
- Category line — e.g. "Product testing · Awaiting your review"
- Meta row (icon + label):
  - `{pool} XLM Pool`
  - `{n} XLM Per winner`
  - `{n} responses`
  - Time line: "Expires in {x}" or "Expired {date}"
- Row action: **"Review responses"** (primary button)
- Overflow menu (`⋯`): Pause quest, Duplicate, Delete, Share link

### 6.3 Drafted — Populated
- Same row anatomy; status pill: "Draft" (neutral/gray pill)
- Category line reflects draft context, e.g. "Product testing · Incomplete setup"
- Meta row: pool/per-winner values shown as configured (or "—" if unset), responses omitted or "0"
- Time line: "Last edited {date}"
- Row action: **"Edit quest"** (primary button)
- Overflow menu (`⋯`): Duplicate, Delete

### 6.4 Completed — Populated
- Status pill: "Completed" (green/muted pill)
- Category line, e.g. "Product testing · Winners announced"
- Meta row: final pool, per-winner payout, total responses
- Time line: "Completed {date}"
- Row action: **"View quest"** (secondary/outline button)
- Overflow menu (`⋯`): Duplicate, Export results, Share results

---

## 7. Responsive Row/Card Behavior

- **Desktop:** Full horizontal row — thumbnail, title/meta block, action button, overflow menu all in one line
- **Tablet:** Row compresses; meta row may wrap to a second line under title while action + overflow stay right-aligned
- **Mobile:** Row becomes a stacked card — thumbnail + title/status pill on top, category line below, meta row wraps as a 2x2 or scrollable chip row, action button full-width at the bottom of the card, overflow accessible via a top-right `⋯` on the card


---

## 8. Frame Map (Figma File Structure)

1. **Cover page** — component inventory + asset export list (this document, visualized)
2. **Desktop — Quests hub** — chrome + header + tabs (populated Active)
3. **Desktop — Empty Active state**
4. **Desktop — Populated Active / Drafted / Completed** (3 frames, row actions differ per tab)
5. **Tablet — Quests hub** (empty + populated variants)
6. **Mobile — Quests hub** (empty + populated variants, stacked card behavior)

---


## 9. Submission

- **Branch:** `design/creator-quests`
