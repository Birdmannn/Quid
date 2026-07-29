"use client";

import { useState } from "react";
import Link from "next/link";
import QuestRow from "./QuestRow";
import EmptyActiveQuests from "./EmptyActiveQuests";
import type { QuestRowData, QuestTabKey } from "./types";

// Fixture data until GET /missions/me is wired up.
const QUESTS: QuestRowData[] = [
  {
    id: "1",
    title: "Download and test the latest Ruze.stellar 2.0",
    tagLabel: "Reviewing submissions",
    tagVariant: "active",
    category: "Product testing",
    pool: 610,
    perWinner: 10,
    responses: 72,
    meta: "Expired 2d ago",
    tab: "active",
  },
  {
    id: "2",
    title: "Untitled quest",
    tagLabel: "Draft",
    tagVariant: "draft",
    category: "Product testing",
    pool: 1250,
    perWinner: 52,
    responses: 0,
    meta: "Last edited 8m ago",
    tab: "drafted",
  },
  {
    id: "3",
    title: "Creator onboarding research",
    tagLabel: "Draft",
    tagVariant: "draft",
    category: "Community participation",
    pool: 0,
    perWinner: 0,
    responses: 0,
    meta: "Last edited 2d ago",
    tab: "drafted",
  },
  {
    id: "4",
    title: "Wallet transfer flow feedback",
    tagLabel: "Completed",
    tagVariant: "completed",
    category: "Product testing",
    pool: 1250,
    perWinner: 52,
    responses: 40,
    meta: "Completed Jun 6",
    tab: "completed",
  },
];

const TABS: { key: QuestTabKey; label: string }[] = [
  { key: "active", label: "Active Quest" },
  { key: "drafted", label: "Drafted" },
  { key: "completed", label: "Completed" },
];

export default function QuestsHub() {
  const [activeTab, setActiveTab] = useState<QuestTabKey>("active");

  const counts: Record<QuestTabKey, number> = {
    active: QUESTS.filter((quest) => quest.tab === "active").length,
    drafted: QUESTS.filter((quest) => quest.tab === "drafted").length,
    completed: QUESTS.filter((quest) => quest.tab === "completed").length,
  };

  const visibleQuests = QUESTS.filter((quest) => quest.tab === activeTab);

  return (
    <div>
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-xl font-semibold text-white">Quests</h1>
          <p className="mt-1 text-sm text-white/50">
            Create, manage and review your participation campaigns.
          </p>
        </div>
        <Link
          href="/creator/quests/new"
          className="inline-flex shrink-0 items-center justify-center rounded-lg bg-[#8B5CF6] px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-[#7c0de0]"
        >
          + Add new Quest
        </Link>
      </div>

      <div
        className="flex gap-6 border-b border-white/10 text-sm font-medium text-white/40"
        role="tablist"
        aria-label="Quest status"
      >
        {TABS.map((tab) => {
          const isActive = activeTab === tab.key;

          return (
            <button
              key={tab.key}
              type="button"
              role="tab"
              aria-selected={isActive}
              onClick={() => setActiveTab(tab.key)}
              className={`relative flex items-center gap-1.5 pb-3 transition-colors hover:text-white ${
                isActive ? "text-[#B78CFF]" : ""
              }`}
            >
              {tab.label}
              <span
                className={`rounded-full px-1.5 py-0.5 text-xs ${
                  isActive
                    ? "bg-[#8B5CF6]/20 text-[#B78CFF]"
                    : "bg-white/5 text-white/40"
                }`}
              >
                {counts[tab.key]}
              </span>
              {isActive ? (
                <span className="absolute inset-x-0 bottom-[-1px] h-0.5 bg-[#8B5CF6]" />
              ) : null}
            </button>
          );
        })}
      </div>

      <div className="pt-6">
        {visibleQuests.length === 0 ? (
          activeTab === "active" ? (
            <EmptyActiveQuests />
          ) : (
            <p className="py-12 text-center text-sm text-white/40">
              No {activeTab} quests yet.
            </p>
          )
        ) : (
          <div className="divide-y divide-white/5">
            {visibleQuests.map((quest) => (
              <QuestRow key={quest.id} quest={quest} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
