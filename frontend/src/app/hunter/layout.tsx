import Sidebar from "@/components/hunter/Sidebar";
import RequireWallet from "@/components/hunter/RequireWallet";
import TopNav from "@/components/hunter/TopNav";

export default function HunterLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <RequireWallet>
      <div className="flex h-screen bg-[#0D0B10] text-white">
        <Sidebar />
        <div className="flex min-w-0 flex-1 flex-col">
          <TopNav />
          <main className="flex-1 overflow-y-auto">{children}</main>
        </div>
      </div>
    </RequireWallet>
  );
}
