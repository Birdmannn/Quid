import Image from "next/image";
import HunterQuestTabs from "@/components/hunter/HunterQuestTabs";

const earners = [
  ["Samuel", "Download and test the lat...", "10 XLM"],
  ["James", "Review and implement fe...", "5 XLM"],
  ["Nat.sol", "Finalize marketing strateg...", "12 XLM"],
  ["Saint", "Review and implement fe...", "5 XLM"],
  ["Chioma.stel", "Download and test the lat...", "10 XLM"],
  ["St.sammi", "Finalize marketing strateg...", "12 XLM"],
];

const submissions = [
  ["/dashboard/recent1.svg", "Samuel", "Download and test the lat...", "2 min"],
  ["/dashboard/recent2.svg", "Alice", "Review the project docu...", "5 min"],
  ["/dashboard/recent3.svg", "Michael", "Attend the design feedba...", "30 min"],
  ["/dashboard/recent1.svg", "Jessica", "Finalize wireframes for ap...", "15 min"],
];

const stats = [
  ["Missions completed", "18"],
  ["Pending", "2"],
  ["Earned", "4,870 XLM"],
];

export default function HunterDashboard() {
  return (
    <div className="min-h-screen bg-[#0D0B10] text-white">
      <section className="bg-[linear-gradient(105deg,#7466B0_0%,#3A345D_46%,#111016_100%)]">
        <div className="mx-auto max-w-[1720px] px-5 py-16 sm:px-8 sm:py-18 lg:px-12">
          <h1 className="text-2xl font-semibold tracking-normal sm:text-5xl">
            Welcome back, Samuel.
          </h1>
          <p className="mt-6 text-xl text-white/90 sm:text-xl">
            Take a quest to start earning
          </p>
        </div>
      </section>

      <main className="mx-auto grid max-w-[1720px]  px-5 gap-5 sm:px-8 lg:grid-cols-[1fr_420px] lg:px-12">
        <section className="">
          {/* <div className="grid gap-3 border-b border-white/10 pb-5 sm:grid-cols-3">
            {stats.map(([label, value]) => (
              <div key={label} className="rounded-lg border border-white/10 bg-white/[0.03] p-4">
                <p className="text-sm text-white/50">{label}</p>
                <p className="mt-2 text-2xl font-semibold">{value}</p>
              </div>
            ))}
          </div> */}

          <HunterQuestTabs />
        </section>

        <aside className="border-white/10 lg:border-l lg:pl-8">
          <section className="mt-6">
            <h2 className="text-xl font-semibold">Recent Earners</h2>
            <div className="mt-7 space-y-6">
              {earners.map(([name, mission, amount]) => (
                <div key={`${name}-${amount}`} className="grid grid-cols-[34px_1fr_auto] items-center gap-3">
                  <Image
                    src="/dashboard/avatar.png"
                    alt=""
                    width={34}
                    height={34}
                    className="rounded-full"
                  />
                  <div className="min-w-0">
                    <p className="truncate text-sm text-white/50">{name}</p>
                    <p className="truncate text-sm">{mission}</p>
                  </div>
                  <p className="font-bold">{amount}</p>
                </div>
              ))}
            </div>
          </section>

          <section className="mt-10 border-t border-white/10 pt-6 pb-10">
            <h2 className="text-xl font-semibold">Recently Submitted</h2>
            <div className="mt-7 space-y-6">
              {submissions.map(([icon, name, mission, time]) => (
                <div key={`${name}-${time}`} className="grid grid-cols-[64px_1fr_auto] items-center gap-3">
                  <Image
                    src={icon}
                    alt=""
                    width={64}
                    height={44}
                    className="h-11 w-16 rounded-md object-cover"
                  />
                  <div className="min-w-0">
                    <p className="truncate text-lg">{name}</p>
                    <p className="truncate text-sm text-white/45">{mission}</p>
                  </div>
                  <p className="text-sm font-semibold">{time}</p>
                </div>
              ))}
            </div>
          </section>
        </aside>
      </main>
    </div>
  );
}
