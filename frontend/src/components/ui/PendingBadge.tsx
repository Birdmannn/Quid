import { Loader2 } from 'lucide-react';

interface PendingBadgeProps {
  /** Optional extra className */
  className?: string;
}

/**
 * Issue #349 – Shown on a quest row immediately after the user signs the
 * on-chain transaction.  Disappears once the indexer catches up and the API
 * reflects the submission.
 */
export function PendingBadge({ className = '' }: PendingBadgeProps) {
  return (
    <span
      role="status"
      aria-live="polite"
      aria-label="Transaction pending – waiting for indexer confirmation"
      className={`inline-flex items-center gap-1.5 rounded-full border border-amber-400/40 bg-amber-400/10 px-2.5 py-1 text-xs font-semibold text-amber-300 ${className}`}
    >
      <Loader2 className="size-3 animate-spin" aria-hidden="true" />
      Pending
    </span>
  );
}
