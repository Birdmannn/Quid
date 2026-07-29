export type QuestType = "product-testing" | "community-participation";

export interface BasicsData {
  questType: QuestType;
  title: string;
  description: string;
  completionDuration: string;
  participantLimit: number;
  visibility: "public" | "private";
}

export interface EligibilityData {
  whoCanParticipate: string;
  maxSubmissionsPerParticipant: number;
  eligibleRegions: string;
  previousWinnersCanParticipate: boolean;
  additionalRequirement: string;
}

export type TaskResponseType = "paragraph" | "short-answer" | "upload" | "link";

export interface TaskBlock {
  id: string;
  responseType: TaskResponseType;
  title: string;
  instruction: string;
  required: boolean;
}

export interface RewardsData {
  rewardMethod: string;
  totalRewardBudget: number;
  numberOfWinners: number;
}

export interface ScheduleData {
  publishOption: "immediately" | "scheduled";
  startDateTime: string;
  closingDateTime: string;
  timeZone: string;
  winnerAnnouncement: string;
  autoCloseWhenFull: boolean;
}

export interface QuestWizardData {
  basics: BasicsData;
  eligibility: EligibilityData;
  tasks: TaskBlock[];
  rewards: RewardsData;
  schedule: ScheduleData;
}

export type WizardStepKey =
  | "basics"
  | "eligibility"
  | "tasks"
  | "rewards"
  | "schedule"
  | "review";

export const WIZARD_STEPS: { key: WizardStepKey; label: string }[] = [
  { key: "basics", label: "Basics" },
  { key: "eligibility", label: "Eligibility" },
  { key: "tasks", label: "Tasks" },
  { key: "rewards", label: "Rewards" },
  { key: "schedule", label: "Schedule" },
  { key: "review", label: "Review & publish" },
];

export const AVAILABLE_WALLET_BALANCE = 1240;
export const PLATFORM_FEE_RATE = 0.00375;
export const NETWORK_FEE_ESTIMATE = 0.1;

let taskIdCounter = 0;
export function createTaskBlock(overrides: Partial<TaskBlock> = {}): TaskBlock {
  taskIdCounter += 1;
  return {
    id: `task-${Date.now()}-${taskIdCounter}`,
    responseType: "paragraph",
    title: "",
    instruction: "",
    required: false,
    ...overrides,
  };
}

export function createDefaultWizardData(): QuestWizardData {
  const now = new Date();
  const inOneWeek = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000);

  return {
    basics: {
      questType: "product-testing",
      title: "",
      description: "",
      completionDuration: "10 mins",
      participantLimit: 100,
      visibility: "public",
    },
    eligibility: {
      whoCanParticipate: "Anyone with an eligible account",
      maxSubmissionsPerParticipant: 1,
      eligibleRegions: "All",
      previousWinnersCanParticipate: false,
      additionalRequirement: "",
    },
    tasks: [
      createTaskBlock({ responseType: "paragraph" }),
      createTaskBlock({ responseType: "upload" }),
    ],
    rewards: {
      rewardMethod: "Selected winners",
      totalRewardBudget: 640,
      numberOfWinners: 64,
    },
    schedule: {
      publishOption: "immediately",
      startDateTime: toDateTimeLocal(now),
      closingDateTime: toDateTimeLocal(inOneWeek),
      timeZone: "UTC",
      winnerAnnouncement: toDateTimeLocal(inOneWeek),
      autoCloseWhenFull: false,
    },
  };
}

export function toDateTimeLocal(date: Date): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function formatDateTime(value: string): string {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export function rewardCalculations(rewards: RewardsData) {
  const platformFee = rewards.totalRewardBudget * PLATFORM_FEE_RATE;
  const networkFee = NETWORK_FEE_ESTIMATE;
  const totalRequired = rewards.totalRewardBudget + platformFee + networkFee;
  const remaining = AVAILABLE_WALLET_BALANCE - totalRequired;
  const rewardPerWinner =
    rewards.numberOfWinners > 0
      ? rewards.totalRewardBudget / rewards.numberOfWinners
      : 0;

  return {
    platformFee,
    networkFee,
    totalRequired,
    remaining,
    rewardPerWinner,
  };
}
