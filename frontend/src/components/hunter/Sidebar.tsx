"use client";

import { useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { BiSolidWallet } from "react-icons/bi";
import { HiMenu, HiX } from "react-icons/hi";
import { RiFileList3Line } from "react-icons/ri";
import { TbLayoutDashboard } from "react-icons/tb";
import { LuClipboardCheck } from "react-icons/lu";

const navItems = [
  {
    href: "/hunter",
    label: "Dashboard",
    icon: TbLayoutDashboard,
  },
  {
    href: "/hunter/mission-board",
    label: "Mission Board",
    icon: RiFileList3Line,
  },
  {
    href: "/hunter/my-submissions",
    label: "My Submissions",
    icon: LuClipboardCheck,
  },
  {
    href: "/hunter/wallet",
    label: "Wallet",
    icon: BiSolidWallet,
  },
];

export default function Sidebar() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);

  const linkClasses = (href: string) => {
    const isActive =
      pathname === href || (href !== "/hunter" && pathname.startsWith(href));

    return `flex items-center gap-2 border-l-[3px] py-2 pl-3 transition-colors ${
      isActive
        ? "border-[#9011FF] text-white"
        : "border-transparent text-[#8C86B8] hover:text-white"
    }`;
  };

  return (
    <>
      <button
        type="button"
        onClick={() => setIsOpen((open) => !open)}
        className="fixed right-4 top-4 z-50 rounded-lg bg-[#141026] p-2 text-white md:hidden"
        aria-label={isOpen ? "Close hunter navigation" : "Open hunter navigation"}
      >
        {isOpen ? <HiX size={24} /> : <HiMenu size={24} />}
      </button>

      {isOpen && (
        <button
          type="button"
          className="fixed inset-0 z-30 bg-black/50 md:hidden"
          onClick={() => setIsOpen(false)}
          aria-label="Close hunter navigation overlay"
        />
      )}

      <aside
        className={`fixed left-0 top-0 z-40 flex h-full w-64 flex-col border-r border-[#241B4A] bg-[#141026] p-4 transition-transform duration-300 md:relative md:h-screen md:translate-x-0 ${
          isOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <div className="mb-8 flex items-center justify-between border-b border-b-[#241B4A] p-4">
          <Image src="/logo.png" alt="Quid" width={55} height={32} />
          <h1 className="text-sm font-bold text-[#CFC9FF]">Hunters</h1>
        </div>

        <nav className="flex flex-col gap-4">
          {navItems.map((item) => {
            const Icon = item.icon;

            return (
              <Link
                key={item.href}
                href={item.href}
                className={linkClasses(item.href)}
                onClick={() => setIsOpen(false)}
              >
                <Icon size={18} />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>

        <div className="mt-auto rounded-lg bg-[#1B1540] p-4 text-sm text-[#CFC9FF]">
          <p className="font-semibold text-white">Hunter mode</p>
          <p className="mt-2 text-xs text-[#8C86B8]">
            Complete missions and track every XLM reward.
          </p>
        </div>
      </aside>
    </>
  );
}
