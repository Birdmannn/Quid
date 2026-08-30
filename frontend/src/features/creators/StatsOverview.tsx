import React from "react";
import Image from "next/image";
import UserIcon from "../../../public/statsoverview/User.png";
import PaperIcon from "../../../public/statsoverview/Paper.png";
import WalletIcon from "../../../public/statsoverview/Wallet.png";

interface StatsOverviewProps {
  activeQuests: number;
  totalResponses: number;
  totalRewards: number;
  onCreateQuest?: () => void;
}

export const StatsOverview: React.FC<StatsOverviewProps> = ({
  activeQuests,
  totalResponses,
  totalRewards,
  onCreateQuest,
}) => {
  const stats = [
    {
      icon: PaperIcon,
      label: "Active Quests",
      value: activeQuests,
      bgColor: "bg-purple-500/10",
      glowColor: "drop-shadow-[0_0_8px_rgba(168,85,247,0.6)]",
    },
    {
      icon: UserIcon,
      label: "Total response",
      value: totalResponses,
      bgColor: "bg-blue-500/10",
      glowColor: "drop-shadow-[0_0_8px_rgba(59,130,246,0.6)]",
    },
    {
      icon: WalletIcon,
      label: "Total rewards",
      value: `${totalRewards.toLocaleString("en-US", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })} USD`,
      bgColor: "bg-yellow-500/10",
      glowColor: "drop-shadow-[0_0_8px_rgba(234,179,8,0.6)]",
    },
  ];

  return (
    <div className="mb-8 grid grid-cols-1 items-center gap-6 lg:grid-cols-3 lg:gap-8">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3 lg:col-span-2">
        {stats.map((stat) => (
          <div
            key={stat.label}
            className="flex items-center gap-3 rounded-xl bg-white/[0.03] px-5 py-5"
          >
            <div className={`${stat.bgColor} rounded-lg p-2.5`}>
              <Image
                src={stat.icon}
                alt={stat.label}
                width={20}
                height={20}
                className={`h-5 w-5 ${stat.glowColor}`}
              />
            </div>
            <div className="min-w-0">
              <p className="truncate text-3xl font-bold text-white">
                {typeof stat.value === "number"
                  ? stat.value
                  : stat.value.split(" ")[0]}
              </p>
              <p className="mt-0.5 text-xs text-white/50">{stat.label}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="flex justify-end lg:col-span-1">
        <button
          type="button"
          onClick={onCreateQuest}
          className="flex w-full items-center justify-center rounded-xl bg-[#9011FF] px-6 py-3 text-base font-semibold text-white shadow-lg shadow-purple-500/30 transition-all duration-200 hover:bg-purple-700 hover:shadow-purple-500/50 sm:w-auto"
        >
          Create a New Survey
        </button>
      </div>
    </div>
  );
};
