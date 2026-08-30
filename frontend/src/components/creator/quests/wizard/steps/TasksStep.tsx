import { ChevronDown, ChevronUp, GripVertical, Plus, Trash2 } from "lucide-react";
import { Checkbox, FieldLabel, SelectField, TextArea, TextInput } from "../fields";
import { createTaskBlock, type TaskBlock, type TaskResponseType } from "../types";

const RESPONSE_TYPE_LABELS: Record<TaskResponseType, string> = {
  paragraph: "Paragraph",
  "short-answer": "Short answer",
  upload: "Upload",
  link: "Link",
};

const RESPONSE_TYPE_OPTIONS = Object.values(RESPONSE_TYPE_LABELS);

function labelToResponseType(label: string): TaskResponseType {
  const entry = Object.entries(RESPONSE_TYPE_LABELS).find(
    ([, value]) => value === label,
  );
  return (entry?.[0] as TaskResponseType) ?? "paragraph";
}

export default function TasksStep({
  tasks,
  onChange,
}: {
  tasks: TaskBlock[];
  onChange: (tasks: TaskBlock[]) => void;
}) {
  const updateTask = (id: string, patch: Partial<TaskBlock>) => {
    onChange(tasks.map((task) => (task.id === id ? { ...task, ...patch } : task)));
  };

  const removeTask = (id: string) => {
    onChange(tasks.filter((task) => task.id !== id));
  };

  const moveTask = (index: number, direction: -1 | 1) => {
    const targetIndex = index + direction;
    if (targetIndex < 0 || targetIndex >= tasks.length) return;
    const next = [...tasks];
    [next[index], next[targetIndex]] = [next[targetIndex], next[index]];
    onChange(next);
  };

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-white/50">
        Use clear questions and only request evidence that is useful for
        review.
      </p>

      {tasks.map((task, index) => (
        <div
          key={task.id}
          className="rounded-xl border border-white/10 bg-[#100D1C]/60 p-4"
        >
          <div className="mb-4 flex items-center justify-between gap-2">
            <div className="flex items-center gap-2 text-xs font-semibold tracking-wide text-white/50">
              <GripVertical className="size-4 text-white/30" />
              TASK {index + 1}
              <div className="flex flex-col">
                <button
                  type="button"
                  aria-label="Move task up"
                  disabled={index === 0}
                  onClick={() => moveTask(index, -1)}
                  className="text-white/30 hover:text-white disabled:cursor-not-allowed disabled:opacity-30"
                >
                  <ChevronUp className="size-3.5" />
                </button>
                <button
                  type="button"
                  aria-label="Move task down"
                  disabled={index === tasks.length - 1}
                  onClick={() => moveTask(index, 1)}
                  className="text-white/30 hover:text-white disabled:cursor-not-allowed disabled:opacity-30"
                >
                  <ChevronDown className="size-3.5" />
                </button>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <div className="w-36">
                <SelectField
                  id={`task-type-${task.id}`}
                  value={RESPONSE_TYPE_LABELS[task.responseType]}
                  onChange={(label) =>
                    updateTask(task.id, { responseType: labelToResponseType(label) })
                  }
                  options={RESPONSE_TYPE_OPTIONS}
                />
              </div>
              <button
                type="button"
                aria-label="Remove task"
                onClick={() => removeTask(task.id)}
                className="flex size-9 shrink-0 items-center justify-center rounded-md text-white/40 transition-colors hover:bg-white/5 hover:text-red-400"
              >
                <Trash2 className="size-4" />
              </button>
            </div>
          </div>

          <div className="mb-4">
            <div className="mb-2 flex items-center justify-between">
              <label
                htmlFor={`task-title-${task.id}`}
                className="text-sm font-medium text-white"
              >
                Task title
              </label>
              <Checkbox
                checked={task.required}
                onChange={(required) => updateTask(task.id, { required })}
                label="Required task"
              />
            </div>
            <TextInput
              id={`task-title-${task.id}`}
              value={task.title}
              onChange={(title) => updateTask(task.id, { title })}
              placeholder="e.g. Share your feedback"
            />
          </div>

          <div>
            <FieldLabel htmlFor={`task-instruction-${task.id}`}>
              Participant instruction
            </FieldLabel>
            <TextArea
              id={`task-instruction-${task.id}`}
              value={task.instruction}
              onChange={(instruction) => updateTask(task.id, { instruction })}
              placeholder="Explain exactly what participants should submit."
            />
          </div>
        </div>
      ))}

      <button
        type="button"
        onClick={() => onChange([...tasks, createTaskBlock()])}
        className="flex items-center justify-center gap-1.5 self-start rounded-lg border border-white/15 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-white/5"
      >
        <Plus className="size-4" />
        Add task block
      </button>
    </div>
  );
}

export function isTasksStepValid(tasks: TaskBlock[]): boolean {
  return tasks.length > 0 && tasks.every((task) => task.title.trim().length > 0);
}
