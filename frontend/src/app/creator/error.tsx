"use client";

import { useEffect } from "react";
import Link from "next/link";

export default function CreatorError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error("Creator dashboard error:", error);
  }, [error]);

  return (
    <div className="flex h-screen items-center justify-center bg-[#0D0B10] text-white px-4">
      <div className="mx-auto max-w-md text-center">
        <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10">
          <svg
            className="h-8 w-8 text-red-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        </div>

        <h2 className="mb-2 text-xl font-semibold sm:text-2xl">
          Something went wrong
        </h2>
        <p className="mb-8 text-sm text-[#8C86B8] sm:text-base">
          The creator dashboard hit an unexpected error. You can try again or go
          back to the home page.
        </p>

        <div className="flex flex-col items-center justify-center gap-3 sm:flex-row sm:gap-4">
          <button
            type="button"
            onClick={reset}
            className="inline-flex items-center justify-center rounded-lg bg-[#9011FF] px-6 py-3 text-sm font-medium text-white transition-all hover:bg-[#7a0edb] focus:outline-none focus:ring-2 focus:ring-[#9011FF] focus:ring-offset-2 focus:ring-offset-[#0D0B10]"
          >
            Try again
          </button>
          <Link
            href="/"
            className="inline-flex items-center justify-center rounded-lg border border-white/10 bg-white/[0.03] px-6 py-3 text-sm font-medium text-white transition-all hover:border-white/20 hover:bg-white/[0.06] focus:outline-none focus:ring-2 focus:ring-[#9011FF] focus:ring-offset-2 focus:ring-offset-[#0D0B10]"
          >
            Back to home
          </Link>
        </div>
      </div>
    </div>
  );
}
