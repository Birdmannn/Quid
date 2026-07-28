"use client";

import { useState } from "react";
import QuestItem from "./QuestItem";

type Tab = "available" | "completed";

const tabs: { label: string; value: Tab }[] = [
  { label: "Available Quest", value: "available" },
  { label: "Completed", value: "completed" },
];

const availableQuests = [
  {
    id: "1",
    title: "Download and test the latest Ruze.stellar 2.0",
    tag: "Awaiting Review",
    category: "Product",
    reward: "10 XLM (640)",
    deadline: "Expired 2d ago",
    score: 72,
  },
  {
    id: "2",
    title: "Review and implement feedback for Ruze.stellar 2.0",
    category: "Development",
    reward: "10 XLM (640)",
    deadline: "Due in 4d",
    score: 23,
  },
  {
    id: "3",
    title: "Finalize marketing strategy for Ruze.stellar 2.0",
    category: "Marketing",
    reward: "10 XLM (640)",
    deadline: "Due in 14d",
    score: 11,
  },
  {
    id: "4",
    title: "Prepare launch event for Ruze.stellar 2.0",
    category: "Events",
    reward: "10 XLM (640)",
    deadline: "Due in 16d",
    score: 8,
  },
];

function CompletedState() {
  return (
    <div className="py-12 text-center text-white/50">
      <p>No completed quests yet.</p>
    </div>
  );
}

export default function Questlist() {
  const [activeTab, setActiveTab] = useState<Tab>("available");

  return (
    <div>
      <div
        className="flex gap-8 border-b border-white/10 text-lg font-semibold text-white/40"
        role="tablist"
        aria-label="Quest filters"
      >
        {tabs.map((tab) => {
          const isActive = activeTab === tab.value;

          return (
            <button
              key={tab.value}
              type="button"
              role="tab"
              aria-selected={isActive}
              onClick={() => setActiveTab(tab.value)}
              className={`relative pb-4 transition-colors hover:text-white ${
                isActive ? "text-[#B78CFF]" : ""
              }`}
            >
              {tab.label}
              {isActive ? (
                <span className="absolute inset-x-0 bottom-[-1px] h-0.5 bg-[#B78CFF]" />
              ) : null}
            </button>
          );
        })}
      </div>

      <div className="flex flex-col gap-9 pt-7 pb-12">
        {activeTab === "available" ? (
          availableQuests.map((quest) => (
            <QuestItem key={quest.id} {...quest} />
          ))
        ) : (
          <CompletedState />
        )}
      </div>
    </div>
  );
}
