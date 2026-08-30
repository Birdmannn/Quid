"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import QuestsHub from "@/components/creator/quests/QuestsHub";
import QuestWizard from "@/components/creator/quests/wizard/QuestWizard";
import PublishSuccess from "@/components/creator/quests/wizard/PublishSuccess";

export default function CreateQuestPage() {
  const router = useRouter();
  const [publishedQuestId, setPublishedQuestId] = useState<string | null>(null);

  if (publishedQuestId) {
    return <PublishSuccess questId={publishedQuestId} />;
  }

  return (
    <>
      <div className="h-[calc(100vh-5rem)] overflow-y-auto px-5 py-4 sm:px-8 sm:py-6 lg:px-12">
        <QuestsHub />
      </div>

      <div className="fixed inset-0 z-50">
        <button
          type="button"
          aria-label="Close quest creator"
          onClick={() => router.push("/creator/quests")}
          className="absolute inset-0 bg-black/60 backdrop-blur-[1px]"
        />

        <div className="absolute inset-y-0 right-0 w-full border-l border-white/10 bg-[#0B0913] shadow-2xl sm:w-[45%]">
          <QuestWizard onPublish={setPublishedQuestId} />
        </div>
      </div>
    </>
  );
}
