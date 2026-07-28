import Link from "next/link";

export default function EmptyActiveQuests() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-24 text-center">
      <h2 className="text-lg font-semibold text-white">No Active Quest</h2>
      <p className="max-w-xs text-sm text-white/50">
        You have no active quest at the moment, click the button below to
        create a quest.
      </p>
      <Link
        href="/creator/quests/new"
        className="mt-2 rounded-lg bg-[#9011FF] px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-[#7c0de0]"
      >
        + Add new Quest
      </Link>
    </div>
  );
}
