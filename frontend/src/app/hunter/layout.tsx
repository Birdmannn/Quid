"use client";

import Sidebar from "@/components/hunter/Sidebar";
import RequireWallet from "@/components/auth/RequireWallet";
import RequireRole from "@/components/auth/RequireRole";
import TopNav from "@/components/hunter/TopNav";

export default function HunterLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <RequireWallet>
      {/* Issue #331: the wallet gate says who you are, this says which
          dashboard is yours - server role first, local role as the fallback. */}
      <RequireRole role="hunter">
        <div className="flex h-screen bg-[#0D0B10] text-white">
          <Sidebar />
          <div className="flex min-w-0 flex-1 flex-col">
            <TopNav />
            <main className="flex-1 overflow-y-auto">{children}</main>
          </div>
        </div>
      </RequireRole>
    </RequireWallet>
  );
}
