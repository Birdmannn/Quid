import Image from "next/image";
import Link from "next/link";
import briefcaseIcon from "../../../public/quest-detail/briefcase-icon.png";
import stellarIcon from "../../../public/quest-detail/stellar-icon.png";

export default function QuestItem({
  title,
  tag,
  category,
  reward,
  deadline,
  score,
  id,
}: {
  title: string;
  tag?: string;
  category: string;
  reward: string;
  deadline: string;
  score: number;
  id: string;
}) {
  return (
    <Link
      href={`/creator/quests/${id}`}
      className="group grid gap-5 sm:grid-cols-[96px_1fr_auto] sm:items-center"
    >
      <Image
        src="/namelogo.png"
        alt=""
        width={96}
        height={96}
        className="size-20 rounded-lg object-cover sm:size-24"
      />

      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="truncate text-2xl font-semibold text-white transition-colors group-hover:text-[#B78CFF]">
            {title}
          </h2>
          {tag ? (
            <span className="rounded-lg bg-[#9011FF] px-2 py-1 text-xs text-white">
              {tag}
            </span>
          ) : null}
        </div>
        <div className="mt-2 flex flex-wrap items-center gap-x-8 gap-y-2 text-sm text-white/65">
          <span className="flex items-center gap-2">
            <Image
              src={briefcaseIcon}
              alt=""
              width={14}
              height={14}
              className="h-3.5 w-3.5"
            />
            {category}
          </span>
          <span className="flex items-center gap-2">
            <Image
              src={stellarIcon}
              alt=""
              width={14}
              height={14}
              className="h-3.5 w-3.5"
            />
            {reward}
          </span>
          <span className="text-xs text-[#8C86B8]">{deadline}</span>
        </div>
      </div>

      <div className="flex items-center gap-1.5 text-3xl font-bold text-white">
        <Image
          src={briefcaseIcon}
          alt=""
          width={19}
          height={19}
          className="h-[19px] w-[19px]"
        />
        <span>{score}</span>
      </div>
    </Link>
  );
}
