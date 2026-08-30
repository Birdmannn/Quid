"use client";

import Image from "next/image";
import { useState } from "react";
import { HiBell, HiMenu, HiX } from "react-icons/hi";
import { BiSolidWallet } from "react-icons/bi";
import { RiArrowDropDownLine } from "react-icons/ri";

const QuestHeader = () => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <header className="relative py-5 border-b border-b-[#241B4A] px-4 md:px-0 flex items-center justify-between">
      {/* Mobile Hamburger */}
      <div className="md:hidden">
        <button
          onClick={() => setIsOpen(!isOpen)}
          aria-label={isOpen ? "Close menu" : "Open menu"}
          className="flex items-center justify-center min-h-11 min-w-11 -ml-2"
        >
          {isOpen ? <HiX size={24} /> : <HiMenu size={24} />}
        </button>
      </div>

      {/* Quests Title */}
      <h1 className="flex-1 text-xl font-bold text-center md:text-left md:flex-none">
        Quests
      </h1>

      {/* Desktop Menu */}
      <div className="hidden md:flex items-center gap-4 ml-auto">
        <button>
          <HiBell />
        </button>

        <div className="flex items-center gap-1">
          <BiSolidWallet />
          <span>$0</span>
        </div>

        <div className="flex items-center justify-between p-0.5 gap-1.5">
          <div className="flex items-center gap-[3.5px]">
            <Image src="/namelogo.png" alt="Logo" width={24} height={24} />
            <span>Ruze.stellar</span>
          </div>
          <RiArrowDropDownLine className="w-8 h-8 cursor-pointer" />
        </div>
      </div>

      {/* Mobile Menu Drawer */}
      {isOpen && (
        <>
          {/* Overlay */}
          <div
            className="fixed inset-0 bg-black/50 z-40 md:hidden"
            onClick={() => setIsOpen(false)}
          />
          {/* Drawer */}
          <div className="fixed top-0 right-0 h-full w-64 bg-[#0C0A14] border-l border-l-[#241B4A] z-50 p-4 flex flex-col gap-4 md:hidden shadow-lg">
            <button className="flex items-center gap-2">
              <HiBell /> Notifications
            </button>

            <div className="flex items-center gap-2">
              <BiSolidWallet />
              <span>$0</span>
            </div>

            <div className="flex items-center justify-between p-0.5 gap-1.5">
              <div className="flex items-center gap-[3.5px]">
                <Image src="/namelogo.png" alt="Logo" width={24} height={24} />
                <span>Ruze.stellar</span>
              </div>
              <RiArrowDropDownLine className="w-6 h-6 cursor-pointer" />
            </div>
          </div>
        </>
      )}
    </header>
  );
};

export default QuestHeader;
