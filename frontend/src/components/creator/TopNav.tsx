"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useWallet } from "@/context/WalletProvider";
import { Bell, LogOut } from "lucide-react";
import { useState } from "react";

const topNavItems = [
  {
    href: "/creator",
    label: "Overview",
  },
  {
    href: "/creator/quests",
    label: "Quests",
  },
  {
    href: "/creator/wallet",
    label: "Wallet",
  },
];

function truncateKey(key: string): string {
  if (key.length <= 8) return key;
  return `${key.slice(0, 4)}...${key.slice(-4)}`;
}

export default function TopNav() {
  const pathname = usePathname();
  const { publicKey, disconnect } = useWallet();
  const [showLogout, setShowLogout] = useState(false);

  const isActive = (href: string) =>
    pathname === href ||
    (href !== "/creator" && pathname.startsWith(href));

  return (
    <header className="shrink-0 border-b border-white/10 bg-[#0D0B10]/95 backdrop-blur">
      <div className="flex h-20 items-center justify-between px-5 sm:px-8 lg:px-12">
        <nav className="hidden items-center gap-10 text-sm text-white/70 lg:flex">
          {topNavItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={`transition-colors hover:text-white ${
                isActive(item.href) ? "text-white" : ""
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <div className="ml-auto flex items-center gap-4 text-sm font-semibold">
          <Bell className="hidden size-5 text-white/85 sm:block" />
          <div className="relative">
            <button
              type="button"
              onClick={() => setShowLogout((prev) => !prev)}
              className="flex items-center gap-2 rounded-lg border border-[#241B4A] bg-[#141026] px-3 py-1.5 text-xs font-mono text-[#CFC9FF] hover:border-[#9011FF] transition-colors"
            >
              <span className="h-2 w-2 rounded-full bg-green-400" />
              {publicKey ? truncateKey(publicKey) : ""}
            </button>
            {showLogout && (
              <div className="absolute right-0 mt-2 w-48 rounded-xl border border-[#241B4A] bg-[#141026] p-2 shadow-xl z-50">
                <button
                  type="button"
                  onClick={() => {
                    disconnect();
                    setShowLogout(false);
                  }}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-red-400 hover:bg-red-500/10 transition-colors"
                >
                  <LogOut className="size-4" />
                  Disconnect wallet
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
