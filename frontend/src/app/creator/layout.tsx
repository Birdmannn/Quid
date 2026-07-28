import Sidebar from "@/components/creator/Sidebar";
import TopNav from "@/components/creator/TopNav";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-screen overflow-x-hidden bg-[#0D0B10] text-white">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <TopNav />
        <main className="min-w-0 flex-1 overflow-y-auto bg-[#0D0B10]">
          {children}
        </main>
      </div>
    </div>
  );
}
