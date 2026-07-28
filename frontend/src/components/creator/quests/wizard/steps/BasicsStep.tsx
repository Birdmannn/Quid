import {
  FieldLabel,
  NumberField,
  RadioCard,
  SelectField,
  TextArea,
  TextInput,
} from "../fields";
import type { BasicsData } from "../types";

const DURATION_OPTIONS = ["5 mins", "10 mins", "15 mins", "30 mins", "1 hour"];
const VISIBILITY_OPTIONS = ["Public", "Private"];
const TITLE_MAX_LENGTH = 90;

export default function BasicsStep({
  data,
  onChange,
}: {
  data: BasicsData;
  onChange: (patch: Partial<BasicsData>) => void;
}) {
  return (
    <div className="flex flex-col gap-6">
      <div>
        <FieldLabel>Quest type</FieldLabel>
        <div className="flex flex-col gap-3 sm:flex-row">
          <RadioCard
            title="Product testing"
            description="For product trials, feature testing, bug reports, usability feedback and product reviews."
            selected={data.questType === "product-testing"}
            onSelect={() => onChange({ questType: "product-testing" })}
          />
          <RadioCard
            title="Community participation"
            description="For quests restricted to community members or requiring participants to join a community."
            selected={data.questType === "community-participation"}
            onSelect={() => onChange({ questType: "community-participation" })}
          />
        </div>
      </div>

      <div>
        <FieldLabel
          htmlFor="quest-title"
          hint={`${data.title.length}/${TITLE_MAX_LENGTH}`}
        >
          Quest title
        </FieldLabel>
        <TextInput
          id="quest-title"
          value={data.title}
          maxLength={TITLE_MAX_LENGTH}
          onChange={(title) => onChange({ title })}
          placeholder="e.g. Test the latest version of Rüze.stellar 2.0"
        />
      </div>

      <div>
        <FieldLabel htmlFor="quest-description">Quest description</FieldLabel>
        <TextArea
          id="quest-description"
          value={data.description}
          onChange={(description) => onChange({ description })}
          placeholder="An introduction for participants."
        />
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <div>
          <FieldLabel htmlFor="completion-duration">
            Est. Completion duration
          </FieldLabel>
          <SelectField
            id="completion-duration"
            value={data.completionDuration}
            onChange={(completionDuration) => onChange({ completionDuration })}
            options={DURATION_OPTIONS}
          />
        </div>
        <div>
          <FieldLabel htmlFor="participant-limit">Participant limit</FieldLabel>
          <NumberField
            id="participant-limit"
            value={data.participantLimit}
            onChange={(participantLimit) => onChange({ participantLimit })}
            min={1}
          />
        </div>
        <div>
          <FieldLabel htmlFor="visibility">Visibility</FieldLabel>
          <SelectField
            id="visibility"
            value={data.visibility === "public" ? "Public" : "Private"}
            onChange={(value) =>
              onChange({ visibility: value === "Public" ? "public" : "private" })
            }
            options={VISIBILITY_OPTIONS}
          />
        </div>
      </div>
    </div>
  );
}

export function isBasicsStepValid(data: BasicsData): boolean {
  return data.title.trim().length > 0 && data.description.trim().length > 0;
}
