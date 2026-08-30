"use client";

import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Bell, ChevronDown } from "lucide-react";

const topNavItems = [
  {
    href: "/hunter/mission-board",
    label: "Quest",
  },
  {
    href: "/hunter/my-submissions",
    label: "Bookmark",
  },
  {
    href: "/hunter",
    label: "Leaderboard",
  },
];

export default function TopNav() {
  const pathname = usePathname();

  const isActive = (href: string) =>
    pathname === href || (href !== "/hunter" && pathname.startsWith(href));

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
          <span className="flex items-center gap-1">
            <Image
              src="/dashboard/Star.svg"
              alt=""
              width={18}
              height={18}
              className="h-[18px] w-[18px]"
            />
            5
          </span>
          <span className="hidden items-center gap-1 sm:flex">
            <Image
              src="/dashboard/wallet.svg"
              alt=""
              width={18}
              height={18}
              className="h-[18px] w-[18px]"
            />
            $0
          </span>
          <span className="flex items-center gap-2">
            <Image
              src="/dashboard/avatar.png"
              alt="Samuel"
              width={34}
              height={34}
              className="size-[34px] rounded-full"
            />
            <span className="hidden sm:inline">Samuel</span>
            <ChevronDown className="hidden size-4 sm:block" />
          </span>
        </div>
      </div>
    </header>
  );
}
