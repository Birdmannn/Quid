import { FaPlus, FaFile } from "react-icons/fa";

type QuestAction = {
  title: string;
  icon: React.ReactNode;
};

const actions: QuestAction[] = [
  {
    title: "Create New Quest",
    icon: <FaPlus className="h-9 w-9 text-[#9011FF]" />,
  },
  {
    title: "Create New Quest",
    icon: <FaFile className="h-9 w-9 text-[#9011FF]" />,
  },
  {
    title: "Create New Quest",
    icon: <FaFile className="h-9 w-9 text-[#9011FF]" />,
  },
];

function QuestActionCard({
  title,
  icon,
}: {
  title: string;
  icon: React.ReactNode;
}) {
  return (
    <button type="button" className="flex cursor-pointer flex-col items-center gap-2">
      <div className="flex h-[116px] w-[130px] items-center justify-center rounded-lg bg-[#1B1540] md:h-[136px] md:w-[154px]">
        {icon}
      </div>
      <h2 className="text-sm font-semibold text-white/70">{title}</h2>
    </button>
  );
}

export default function Quests() {
  return (
    <div className="flex flex-wrap items-center justify-center gap-6 py-8 md:justify-start">
      {actions.map((action, index) => (
        <QuestActionCard key={index} {...action} />
      ))}
    </div>
  );
}
