"use client";

import { useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { TbLayoutDashboard } from "react-icons/tb";
import { RiFileList3Line } from "react-icons/ri";
import { BiSolidWallet } from "react-icons/bi";
import { HiMenu, HiX } from "react-icons/hi";

export default function Sidebar() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);

  const linkClasses = (href: string) =>
    `flex items-center gap-2 pl-3 py-2.5 min-h-11 transition-colors
     ${
       pathname === href
         ? "border-l-[3px] border-[#9011FF] text-[#FFFFFF]"
         : "border-l-[3px] border-transparent text-[#8C86B8]"
     }`;

  return (
    <>
      {/* Hamburger button - only visible on mobile */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        aria-label={isOpen ? "Close navigation menu" : "Open navigation menu"}
        className="md:hidden fixed top-4 right-4 z-50 text-white bg-[#141026] p-2.5 rounded-lg flex items-center justify-center min-h-11 min-w-11"
      >
        {isOpen ? <HiX size={24} /> : <HiMenu size={24} />}
      </button>

      {/* Overlay - only visible on mobile when sidebar is open */}
      {isOpen && (
        <div
          className="md:hidden fixed inset-0 bg-black/50 z-30"
          onClick={() => setIsOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside
        className={`
    w-64 max-w-[80vw] sm:w-56 border-r p-4 bg-[#141026] flex flex-col
    md:relative md:translate-x-0 md:h-screen
    fixed top-0 left-0 h-full z-40 transition-transform duration-300
    ${isOpen ? "translate-x-0" : "-translate-x-full"}
  `}
      >
        <div className="p-4 border-b border-b-[#241B4A] flex justify-between mb-8 items-center">
          <Image src="/logo.png" alt="Logo" width={55} height={32} />
          <h1 className="font-bold text-[#CFC9FF] text-[14px]">Creators</h1>
        </div>

        <nav className="flex flex-col gap-4">
          <Link
            href="/creator"
            className={linkClasses("/creator")}
            onClick={() => setIsOpen(false)}
          >
            <TbLayoutDashboard size={18} />
            <span>Dashboard</span>
          </Link>

          <Link
            href="/creator/quests"
            className={linkClasses("/creator/quests")}
            onClick={() => setIsOpen(false)}
          >
            <RiFileList3Line size={18} />
            <span>Quests</span>
          </Link>

          <Link
            href="/creator/wallet"
            className={linkClasses("/creator/wallet")}
            onClick={() => setIsOpen(false)}
          >
            <BiSolidWallet size={18} />
            <span>Wallet</span>
          </Link>
        </nav>
        <div className="mt-auto self-end bg-[#1B1540] rounded-lg m-4 w-[178px] h-[112px]"></div>
      </aside>
    </>
  );
}
