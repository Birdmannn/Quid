import { ChevronDown, ChevronUp, Check } from "lucide-react";

const INPUT_CLASSES =
  "w-full rounded-lg border border-white/10 bg-[#100D1C] px-3 py-2.5 text-sm text-white placeholder:text-white/30 outline-none transition-colors focus:border-[#8B5CF6]/60";

export function FieldLabel({
  children,
  htmlFor,
  hint,
}: {
  children: React.ReactNode;
  htmlFor?: string;
  hint?: string;
}) {
  return (
    <div className="mb-2 flex items-center justify-between">
      <label htmlFor={htmlFor} className="text-sm font-medium text-white">
        {children}
      </label>
      {hint ? <span className="text-xs text-white/40">{hint}</span> : null}
    </div>
  );
}

export function TextInput({
  id,
  value,
  onChange,
  placeholder,
  maxLength,
}: {
  id: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  maxLength?: number;
}) {
  return (
    <input
      id={id}
      type="text"
      value={value}
      maxLength={maxLength}
      onChange={(event) => onChange(event.target.value)}
      placeholder={placeholder}
      className={INPUT_CLASSES}
    />
  );
}

export function TextArea({
  id,
  value,
  onChange,
  placeholder,
  rows = 4,
}: {
  id: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  rows?: number;
}) {
  return (
    <textarea
      id={id}
      value={value}
      rows={rows}
      onChange={(event) => onChange(event.target.value)}
      placeholder={placeholder}
      className={`${INPUT_CLASSES} resize-none`}
    />
  );
}

export function SelectField({
  id,
  value,
  onChange,
  options,
}: {
  id: string;
  value: string;
  onChange: (value: string) => void;
  options: string[];
}) {
  return (
    <div className="relative">
      <select
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className={`${INPUT_CLASSES} appearance-none pr-9`}
      >
        {options.map((option) => (
          <option key={option} value={option} className="bg-[#100D1C]">
            {option}
          </option>
        ))}
      </select>
      <ChevronDown className="pointer-events-none absolute right-3 top-1/2 size-4 -translate-y-1/2 text-white/40" />
    </div>
  );
}

export function NumberField({
  id,
  value,
  onChange,
  min = 0,
  prefixIcon,
}: {
  id: string;
  value: number;
  onChange: (value: number) => void;
  min?: number;
  prefixIcon?: React.ReactNode;
}) {
  const step = (delta: number) => onChange(Math.max(min, value + delta));

  return (
    <div className="relative">
      {prefixIcon ? (
        <span className="pointer-events-none absolute left-3 top-1/2 flex -translate-y-1/2 items-center">
          {prefixIcon}
        </span>
      ) : null}
      <input
        id={id}
        type="number"
        value={value}
        min={min}
        onChange={(event) => onChange(Number(event.target.value))}
        className={`${INPUT_CLASSES} [appearance:textfield] pr-9 ${prefixIcon ? "pl-9" : ""} [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none`}
      />
      <div className="absolute right-2 top-1/2 flex -translate-y-1/2 flex-col">
        <button
          type="button"
          aria-label="Increase"
          onClick={() => step(1)}
          className="text-white/40 hover:text-white"
        >
          <ChevronUp className="size-3.5" />
        </button>
        <button
          type="button"
          aria-label="Decrease"
          onClick={() => step(-1)}
          className="text-white/40 hover:text-white"
        >
          <ChevronDown className="size-3.5" />
        </button>
      </div>
    </div>
  );
}

export function DateTimeField({
  id,
  value,
  onChange,
}: {
  id: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <input
      id={id}
      type="datetime-local"
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className={`${INPUT_CLASSES} [color-scheme:dark]`}
    />
  );
}

export function Checkbox({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2.5 text-sm text-white/80">
      <span
        onClick={() => onChange(!checked)}
        className={`flex size-4 shrink-0 items-center justify-center rounded border transition-colors ${
          checked
            ? "border-[#8B5CF6] bg-[#8B5CF6]"
            : "border-white/20 bg-transparent"
        }`}
      >
        {checked ? <Check className="size-3 text-white" /> : null}
      </span>
      <span onClick={() => onChange(!checked)}>{label}</span>
    </label>
  );
}

export function RadioCard({
  title,
  description,
  selected,
  onSelect,
}: {
  title: string;
  description: string;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={`flex-1 rounded-lg border p-4 text-left transition-colors ${
        selected
          ? "border-[#8B5CF6] bg-[#8B5CF6]/5"
          : "border-white/10 bg-transparent hover:border-white/20"
      }`}
    >
      <div className="mb-1.5 flex items-center justify-between">
        <span className="text-sm font-semibold text-white">{title}</span>
        <span
          className={`flex size-4 shrink-0 items-center justify-center rounded-full border ${
            selected ? "border-[#8B5CF6] bg-[#8B5CF6]" : "border-white/30"
          }`}
        >
          {selected ? <Check className="size-2.5 text-white" /> : null}
        </span>
      </div>
      <p className="text-xs leading-relaxed text-white/45">{description}</p>
    </button>
  );
}
