import { Checkbox, DateTimeField, FieldLabel, SelectField } from "../fields";
import type { ScheduleData } from "../types";

const TIME_ZONE_OPTIONS = ["UTC", "GMT", "EST", "PST", "WAT", "CET"];

export default function ScheduleStep({
  data,
  onChange,
}: {
  data: ScheduleData;
  onChange: (patch: Partial<ScheduleData>) => void;
}) {
  return (
    <div className="flex flex-col gap-6">
      <p className="text-sm text-white/50">
        Pick your campaign timeline — all dates will show up and stay in the
        time zone you choose.
      </p>

      <div className="flex flex-col gap-3 sm:flex-row">
        <button
          type="button"
          onClick={() => onChange({ publishOption: "immediately" })}
          className={`flex-1 rounded-lg border p-4 text-left transition-colors ${
            data.publishOption === "immediately"
              ? "border-[#8B5CF6] bg-[#8B5CF6]/5"
              : "border-white/10 hover:border-white/20"
          }`}
        >
          <p className="text-sm font-semibold text-white">Publish immediately</p>
          <p className="mt-1 text-xs text-white/45">
            Make this quest available when published.
          </p>
        </button>
        <button
          type="button"
          onClick={() => onChange({ publishOption: "scheduled" })}
          className={`flex-1 rounded-lg border p-4 text-left transition-colors ${
            data.publishOption === "scheduled"
              ? "border-[#8B5CF6] bg-[#8B5CF6]/5"
              : "border-white/10 hover:border-white/20"
          }`}
        >
          <p className="text-sm font-semibold text-white">Schedule for later</p>
          <p className="mt-1 text-xs text-white/45">
            Publish automatically at the start date.
          </p>
        </button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div>
          <FieldLabel htmlFor="start-date-time">Start date and time</FieldLabel>
          <DateTimeField
            id="start-date-time"
            value={data.startDateTime}
            onChange={(startDateTime) => onChange({ startDateTime })}
          />
        </div>
        <div>
          <FieldLabel htmlFor="closing-date-time">
            Closing date and time
          </FieldLabel>
          <DateTimeField
            id="closing-date-time"
            value={data.closingDateTime}
            onChange={(closingDateTime) => onChange({ closingDateTime })}
          />
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div>
          <FieldLabel htmlFor="time-zone">Time zone</FieldLabel>
          <SelectField
            id="time-zone"
            value={data.timeZone}
            onChange={(timeZone) => onChange({ timeZone })}
            options={TIME_ZONE_OPTIONS}
          />
        </div>
        <div>
          <FieldLabel htmlFor="winner-announcement">
            Winner announcement
          </FieldLabel>
          <DateTimeField
            id="winner-announcement"
            value={data.winnerAnnouncement}
            onChange={(winnerAnnouncement) => onChange({ winnerAnnouncement })}
          />
        </div>
      </div>

      <Checkbox
        checked={data.autoCloseWhenFull}
        onChange={(autoCloseWhenFull) => onChange({ autoCloseWhenFull })}
        label="Close automatically when the participant limit is reached"
      />
    </div>
  );
}

export function isScheduleStepValid(data: ScheduleData): boolean {
  if (!data.startDateTime || !data.closingDateTime) return false;
  return new Date(data.closingDateTime) > new Date(data.startDateTime);
}
