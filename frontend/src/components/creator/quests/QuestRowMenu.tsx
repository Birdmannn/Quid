"use client";

import { useEffect, useRef, useState } from "react";
import { MoreHorizontal } from "lucide-react";
import type { LucideIcon } from "lucide-react";

interface QuestRowMenuOption {
  label: string;
  icon: LucideIcon;
  onSelect?: () => void;
  danger?: boolean;
}

export default function QuestRowMenu({
  options,
}: {
  options: QuestRowMenuOption[];
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;

    function handleClickOutside(event: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        aria-label="Quest actions"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
        className="flex size-8 items-center justify-center rounded-md text-white/50 transition-colors hover:bg-white/5 hover:text-white"
      >
        <MoreHorizontal className="size-5" />
      </button>

      {open ? (
        <div
          role="menu"
          className="absolute right-0 top-10 z-10 w-48 overflow-hidden rounded-xl border border-white/10 bg-[#100D1C] py-1.5 shadow-xl shadow-black/50"
        >
          {options.map((option) => {
            const Icon = option.icon;

            return (
              <button
                key={option.label}
                type="button"
                role="menuitem"
                onClick={() => {
                  option.onSelect?.();
                  setOpen(false);
                }}
                className={`flex w-full items-center gap-2.5 px-4 py-2.5 text-left text-sm transition-colors hover:bg-white/5 ${
                  option.danger ? "text-red-400" : "text-white/90"
                }`}
              >
                <Icon className="size-4" />
                {option.label}
              </button>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
