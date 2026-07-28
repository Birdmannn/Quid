import Questlist from "@/features/creators/Questlist";
import Quests from "@/features/creators/Quests";

export default function QuestsPage() {
  return (
    <section className="min-h-full px-5 py-4 text-white sm:px-8 sm:py-6 lg:px-12">
      <div className="mb-2 flex items-center justify-between border-b border-white/10 pb-4">
        <h1 className="text-lg font-semibold text-[#B78CFF]">Quests</h1>
      </div>
      <Quests />
      <Questlist />
    </section>
  );
}
