import { Check } from "lucide-react";
import { WIZARD_STEPS } from "./types";

export default function WizardStepper({
  currentIndex,
  maxReachedIndex,
  onStepClick,
}: {
  currentIndex: number;
  maxReachedIndex: number;
  onStepClick: (index: number) => void;
}) {
  return (
    <div className="flex flex-wrap items-center gap-x-2 gap-y-2">
      {WIZARD_STEPS.map((step, index) => {
        const isCompleted = index < maxReachedIndex;
        const isCurrent = index === currentIndex;
        const isReachable = index <= maxReachedIndex;

        return (
          <div key={step.key} className="flex items-center gap-2">
            {index > 0 ? (
              <span className="h-px w-4 shrink-0 bg-white/10 sm:w-6" />
            ) : null}
            <button
              type="button"
              disabled={!isReachable}
              onClick={() => onStepClick(index)}
              className={`flex items-center gap-1.5 ${isReachable ? "cursor-pointer" : "cursor-not-allowed"}`}
            >
              <span
                className={`flex size-5 shrink-0 items-center justify-center rounded-full text-xs font-semibold ${
                  isCompleted
                    ? "bg-[#8B5CF6] text-white"
                    : isCurrent
                      ? "bg-[#8B5CF6] text-white"
                      : "bg-white/10 text-white/40"
                }`}
              >
                {isCompleted ? <Check className="size-3" /> : index + 1}
              </span>
              <span
                className={`whitespace-nowrap text-sm ${
                  isCurrent
                    ? "font-semibold text-white"
                    : isCompleted
                      ? "text-white/70"
                      : "text-white/35"
                }`}
              >
                {step.label}
              </span>
            </button>
          </div>
        );
      })}
    </div>
  );
}
