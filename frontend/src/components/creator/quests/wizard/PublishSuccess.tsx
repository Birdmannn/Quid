import Image from "next/image";
import Link from "next/link";
import { ArrowRight } from "lucide-react";

export default function PublishSuccess({ questId }: { questId: string }) {
  return (
    <div className="flex h-full min-h-[calc(100vh-5rem)] flex-1 flex-col items-center justify-center gap-4 px-4 text-center">
      <Image
        src="/dashboard/star-q.svg"
        alt=""
        width={80}
        height={80}
        className="size-20"
      />
      <h2 className="text-2xl font-semibold text-white">
        Quest Published Successfully
      </h2>
      <p className="max-w-sm text-sm text-white/50">
        Your quest has been successfully added to the campaign
      </p>
      <div className="mt-2 flex items-center gap-3">
        <Link
          href="/creator/quests"
          className="rounded-lg border border-white/15 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-white/5"
        >
          Close
        </Link>
        <Link
          href={`/creator/quests/${questId}`}
          className="flex items-center gap-1.5 rounded-lg bg-[#8B5CF6] px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-[#7c0de0]"
        >
          View quest
          <ArrowRight className="size-4" />
        </Link>
      </div>
    </div>
  );
}
