"use client";

import { useEffect } from "react";
import Image from "next/image";
import { X } from "lucide-react";
import { formatDateTime, rewardCalculations, type QuestWizardData } from "./types";

export default function ParticipantPreviewModal({
  data,
  onClose,
}: {
  data: QuestWizardData;
  onClose: () => void;
}) {
  const { rewardPerWinner } = rewardCalculations(data.rewards);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4">
      <div className="w-full max-w-xl rounded-2xl border border-white/10 bg-[#0D0B10] p-6 shadow-2xl">
        <div className="mb-5 flex items-center justify-between">
          <h2 className="text-sm font-semibold text-white">
            Participant preview
          </h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close preview"
            className="flex size-8 items-center justify-center rounded-md text-white/50 transition-colors hover:bg-white/5 hover:text-white"
          >
            <X className="size-4" />
          </button>
        </div>

        <h3 className="text-xl font-semibold text-white">
          {data.basics.title || "Untitled quest"}
        </h3>
        <div className="mt-2 flex items-center gap-2 text-sm text-white/50">
          <Image
            src="/namelogo.png"
            alt=""
            width={16}
            height={16}
            className="size-4 rounded object-cover"
          />
          Ruze.Stellar
        </div>

        <p className="mt-4 text-sm leading-relaxed text-white/70">
          {data.basics.description || "No description provided yet."}
        </p>

        <div className="mt-5 grid grid-cols-3 gap-4 border-y border-white/10 py-4">
          <div>
            <p className="text-xs text-white/40">Reward</p>
            <p className="mt-1 text-sm font-medium text-white">
              {rewardPerWinner.toLocaleString(undefined, {
                maximumFractionDigits: 2,
              })}{" "}
              XLM
            </p>
          </div>
          <div>
            <p className="text-xs text-white/40">Time</p>
            <p className="mt-1 text-sm font-medium text-white">
              {data.basics.completionDuration}
            </p>
          </div>
          <div>
            <p className="text-xs text-white/40">Closes</p>
            <p className="mt-1 text-sm font-medium text-white">
              {formatDateTime(data.schedule.closingDateTime)}
            </p>
          </div>
        </div>

        <div className="mt-5">
          <p className="mb-3 text-sm font-semibold text-white">
            What you&apos;ll do
          </p>
          <ol className="flex flex-col gap-3">
            {data.tasks.map((task, index) => (
              <li key={task.id} className="flex gap-3">
                <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-[#8B5CF6]/20 text-xs font-semibold text-[#B78CFF]">
                  {index + 1}
                </span>
                <div className="min-w-0">
                  <p className="text-sm text-white">
                    {task.title || "Untitled task"}
                  </p>
                  {task.instruction ? (
                    <p className="text-xs text-white/45">{task.instruction}</p>
                  ) : null}
                </div>
              </li>
            ))}
          </ol>
        </div>
      </div>
    </div>
  );
}
