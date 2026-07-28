"use client";

import { useState, useEffect } from "react";
import { ArrowRight, AlertCircle } from "lucide-react";
import { StatsOverview } from "@/features/creators/StatsOverview";
import { CreatorQuestCard } from "@/features/creators/CreatorQuestCard";
import { ResponsePreview } from "@/features/creators/ResponsePreview";
import { DashboardSkeleton } from "@/features/creators/SkeletonLoaders";
import {
  NoQuestsEmptyState,
  NoResponsesEmptyState,
} from "@/features/creators/DashboardEmptyState";
import { mockQuests, mockResponses, Quest, Response } from "@/features/creators/MockData";

interface DashboardState {
  loading: boolean;
  error: string | null;
  quests: Quest[];
  responses: Response[];
  stats: {
    activeQuests: number;
    totalResponses: number;
    totalRewards: number;
  };
}

const questIcons = [
  "/namelogo.png",
  "/namelogo.png",
  "/namelogo.png",
  "/namelogo.png",
];

export default function CreatorDashboard() {
  const [state, setState] = useState<DashboardState>({
    loading: true,
    error: null,
    quests: [],
    responses: [],
    stats: {
      activeQuests: 0,
      totalResponses: 0,
      totalRewards: 0,
    },
  });

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        setState((prev) => ({ ...prev, loading: true, error: null }));
        await new Promise((resolve) => setTimeout(resolve, 1500));

        setState({
          loading: false,
          error: null,
          quests: mockQuests,
          responses: mockResponses,
          stats: {
            activeQuests: 12,
            totalResponses: 48,
            totalRewards: 2150.02,
          },
        });
      } catch {
        setState((prev) => ({
          ...prev,
          loading: false,
          error: "Failed to load dashboard data. Please try again.",
        }));
      }
    };

    void fetchDashboardData();
  }, []);

  const handleCreateQuest = () => {
    console.log("Create new quest");
  };

  const handleRetry = () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
  };

  if (state.loading) {
    return (
      <div className="min-h-screen text-white flex flex-col">
        <main className="flex-1 px-5 py-6 sm:px-8 lg:px-12">
          <DashboardSkeleton />
        </main>
      </div>
    );
  }

  if (state.error) {
    return (
      <div className="min-h-screen text-white flex flex-col">
        <main className="flex-1 px-5 py-6 sm:px-8 lg:px-12">
          <div className="flex flex-col items-center justify-center px-4 py-12 sm:py-20">
            <AlertCircle className="mb-4 h-12 w-12 text-red-500 sm:h-16 sm:w-16" />
            <h2 className="mb-2 text-center text-2xl font-semibold text-white">
              Error Loading Dashboard
            </h2>
            <p className="mb-6 text-center text-sm text-white/60 sm:text-base">
              {state.error}
            </p>
            <button
              type="button"
              onClick={handleRetry}
              className="rounded-lg bg-purple-600 px-6 py-3 text-sm font-medium text-white transition-all hover:bg-purple-700 sm:text-base"
            >
              Try Again
            </button>
          </div>
        </main>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col text-white">
      <main className="flex-1 px-5 py-4 sm:px-8 sm:py-6 lg:px-12">
        <StatsOverview
          activeQuests={state.stats.activeQuests}
          totalResponses={state.stats.totalResponses}
          totalRewards={state.stats.totalRewards}
          onCreateQuest={handleCreateQuest}
        />

        <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-3 lg:gap-8">
          <div className="lg:col-span-2 lg:border-r lg:border-white/10 lg:pr-8">
            <div className="mb-6 flex items-center justify-between border-b border-white/10 pb-4">
              <h2 className="text-lg font-semibold text-[#B78CFF]">
                Active Quests
              </h2>
              <button
                type="button"
                className="group flex items-center space-x-1 text-sm text-[#B78CFF] transition-colors hover:text-purple-300"
              >
                <span>View all</span>
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
              </button>
            </div>

            {state.quests.length === 0 ? (
              <NoQuestsEmptyState onCreateQuest={handleCreateQuest} />
            ) : (
              <div className="space-y-9">
                {state.quests.map((quest, index) => (
                  <CreatorQuestCard
                    key={quest.id}
                    id={quest.id}
                    title={quest.title}
                    category={quest.category}
                    budget={quest.budget}
                    dueDate={quest.dueDate}
                    submissionCount={quest.submissionCount}
                    icon={questIcons[index % questIcons.length]}
                  />
                ))}
              </div>
            )}
          </div>

          <div className="lg:col-span-1">
            <div className="mb-6 flex items-center justify-between border-b border-white/10 pb-4">
              <h2 className="text-lg font-semibold text-[#B78CFF]">
                Recent Response
              </h2>
              <button
                type="button"
                className="group flex items-center space-x-1 text-sm text-[#B78CFF] transition-colors hover:text-purple-300"
              >
                <span>View all</span>
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
              </button>
            </div>

            {state.responses.length === 0 ? (
              <NoResponsesEmptyState />
            ) : (
              <div className="space-y-3">
                {state.responses.map((response) => (
                  <ResponsePreview
                    key={response.id}
                    id={response.id}
                    respondentName={response.respondentName}
                    respondentAvatar={response.respondentAvatar}
                    questTitle={response.questTitle}
                    timeSinceSubmission={response.timeSinceSubmission}
                  />
                ))}
              </div>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
