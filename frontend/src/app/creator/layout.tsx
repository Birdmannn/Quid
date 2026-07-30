"use client";

import { useWallet } from "@/context/WalletProvider";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import Sidebar from "@/components/creator/Sidebar";
import TopNav from "@/components/creator/TopNav";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { connected, publicKey } = useWallet();
  const router = useRouter();
  const [checked, setChecked] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      if (connected && publicKey) {
        setChecked(true);
      } else {
        router.replace("/connect-wallet");
      }
    }, 500);
    return () => clearTimeout(timer);
  }, [connected, publicKey, router]);

  if (!checked) {
    return (
      <div className="flex h-screen items-center justify-center bg-[#0D0B10] text-white">
        <div className="text-center">
          <div className="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2 border-[#9011FF] border-t-transparent" />
          <p className="text-sm text-[#8C86B8]">Checking wallet connection…</p>
        </div>
      </div>
    );
  }

  if (!connected || !publicKey) {
    return null;
  }

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
