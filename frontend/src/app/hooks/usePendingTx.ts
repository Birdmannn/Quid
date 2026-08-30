'use client';

import { useCallback, useEffect, useRef, useState } from 'react';

export interface PendingEntry {
  /** Local identifier for the quest / mission row */
  questId: string;
  /** On-chain tx hash returned by submitFeedbackToContract */
  txHash: string;
  /** ISO timestamp when the tx was signed – used to expire stale entries */
  signedAt: string;
  /** API submission id once the indexer has caught up, null while pending */
  submissionId: string | null;
}

const STORAGE_KEY = 'quid_pending_txs';
const MAX_AGE_MS = 30 * 60 * 1000; // 30 min – discard if indexer never catches up
const POLL_INTERVAL_MS = 5_000;    // poll every 5 s

function readStorage(): PendingEntry[] {
  if (typeof window === 'undefined') return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as PendingEntry[];
    // Prune stale entries older than MAX_AGE_MS
    const now = Date.now();
    return parsed.filter(
      (e) => now - new Date(e.signedAt).getTime() < MAX_AGE_MS,
    );
  } catch {
    return [];
  }
}

function writeStorage(entries: PendingEntry[]): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // quota exceeded – silently ignore; the badge is best-effort
  }
}

/**
 * Issue #349 – Indexer lag UX with optimistic pending states.
 *
 * Returns:
 *  - `pendingIds` – set of questIds whose tx has been signed but the indexer
 *    hasn't caught up yet. Used to render the "Pending" badge.
 *  - `addPendingTx` – call right after a successful on-chain sign.
 *  - `isPending(id)` – convenience predicate.
 *
 * Internally persists entries to localStorage so a page refresh doesn't wipe
 * the badge, and polls `GET /missions/:id/submissions` (or a lightweight
 * status endpoint when one is available) until the submission appears, then
 * removes the entry so the badge clears.
 */
export function usePendingTx(apiUrl: string, hunterAddress?: string) {
  const [entries, setEntries] = useState<PendingEntry[]>(() => readStorage());
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Persist whenever the entries array changes
  useEffect(() => {
    writeStorage(entries);
  }, [entries]);

  /** Add a pending entry immediately after the user signs the transaction */
  const addPendingTx = useCallback(
    (questId: string, txHash: string) => {
      const entry: PendingEntry = {
        questId,
        txHash,
        signedAt: new Date().toISOString(),
        submissionId: null,
      };
      setEntries((prev) => {
        // Deduplicate by questId – replace if already present
        const without = prev.filter((e) => e.questId !== questId);
        return [...without, entry];
      });
    },
    [],
  );

  /** Remove a pending entry once the API confirms it */
  const clearPendingTx = useCallback((questId: string) => {
    setEntries((prev) => prev.filter((e) => e.questId !== questId));
  }, []);

  /** Returns true when a questId still has an unconfirmed tx */
  const isPending = useCallback(
    (questId: string) => entries.some((e) => e.questId === questId),
    [entries],
  );

  /**
   * Poll the backend for each pending entry. We check the hunter's own
   * submissions list – when the questId appears in the PENDING/APPROVED/PAID
   * status the indexer has caught up and we can clear the optimistic badge.
   */
  useEffect(() => {
    if (entries.length === 0 || !hunterAddress) {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
      return;
    }

    const poll = async () => {
      // Fetch hunter submissions for each pending quest in parallel
      await Promise.allSettled(
        entries.map(async (entry) => {
          try {
            // Try the missions detail endpoint to see if the submission exists
            const url = `${apiUrl}/missions/${entry.questId}/submissions`;
            const res = await fetch(url, {
              headers: hunterAddress
                ? {} // auth header injected by middleware if needed
                : {},
            });
            if (!res.ok) return;

            type SubmissionShape = { hunterAddress: string; id: string };
            const submissions = (await res.json()) as SubmissionShape[];
            const found = submissions.find(
              (s) => s.hunterAddress === hunterAddress,
            );
            if (found) {
              clearPendingTx(entry.questId);
            }
          } catch {
            // network error – keep polling
          }
        }),
      );
    };

    // Run immediately then on interval
    void poll();
    timerRef.current = setInterval(() => void poll(), POLL_INTERVAL_MS);

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [entries, hunterAddress, apiUrl, clearPendingTx]);

  return {
    pendingIds: new Set(entries.map((e) => e.questId)),
    addPendingTx,
    clearPendingTx,
    isPending,
  };
}
