"use client";

import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import {
  Checkbox,
  FieldLabel,
  NumberField,
  SelectField,
  TextArea,
} from "../fields";
import type { EligibilityData } from "../types";

const WHO_CAN_PARTICIPATE_OPTIONS = [
  "Anyone with an eligible account",
  "Members with existing participant history",
];

const REGION_OPTIONS = ["All", "North America", "Europe", "Africa", "Asia"];

export default function EligibilityStep({
  data,
  onChange,
}: {
  data: EligibilityData;
  onChange: (patch: Partial<EligibilityData>) => void;
}) {
  const [advancedOpen, setAdvancedOpen] = useState(true);

  return (
    <div className="flex flex-col gap-6">
      <div>
        <FieldLabel htmlFor="who-can-participate">
          Who can participate?
        </FieldLabel>
        <SelectField
          id="who-can-participate"
          value={data.whoCanParticipate}
          onChange={(whoCanParticipate) => onChange({ whoCanParticipate })}
          options={WHO_CAN_PARTICIPATE_OPTIONS}
        />
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div>
          <FieldLabel htmlFor="max-submissions">
            Maximum submissions per participant
          </FieldLabel>
          <NumberField
            id="max-submissions"
            value={data.maxSubmissionsPerParticipant}
            onChange={(maxSubmissionsPerParticipant) =>
              onChange({ maxSubmissionsPerParticipant })
            }
            min={1}
          />
        </div>
        <div>
          <FieldLabel htmlFor="eligible-regions">
            Eligible countries or regions
          </FieldLabel>
          <SelectField
            id="eligible-regions"
            value={data.eligibleRegions}
            onChange={(eligibleRegions) => onChange({ eligibleRegions })}
            options={REGION_OPTIONS}
          />
        </div>
      </div>

      <Checkbox
        checked={data.previousWinnersCanParticipate}
        onChange={(previousWinnersCanParticipate) =>
          onChange({ previousWinnersCanParticipate })
        }
        label="Previous winners can participate"
      />

      <div>
        <button
          type="button"
          onClick={() => setAdvancedOpen((prev) => !prev)}
          className="flex items-center gap-1.5 text-sm font-medium text-white/70 transition-colors hover:text-white"
        >
          Advanced eligibility
          {advancedOpen ? (
            <ChevronUp className="size-4" />
          ) : (
            <ChevronDown className="size-4" />
          )}
        </button>

        {advancedOpen ? (
          <div className="mt-4">
            <FieldLabel htmlFor="additional-requirement">
              Additional requirement
            </FieldLabel>
            <TextArea
              id="additional-requirement"
              value={data.additionalRequirement}
              onChange={(additionalRequirement) =>
                onChange({ additionalRequirement })
              }
              placeholder="Must create account on @your_platform to be eligible for winning"
            />
          </div>
        ) : null}
      </div>
    </div>
  );
}

export function isEligibilityStepValid(): boolean {
  return true;
}
