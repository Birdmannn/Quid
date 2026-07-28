"use client";

import Image from "next/image";
import { useRouter } from "next/navigation";
import { useState } from "react";
import {
  getDashboardRouteForRole,
  saveUserRole,
  type UserRole,
} from "@/lib/onboarding";

const ROLES: Array<{
  id: UserRole;
  icon: string;
  title: string;
  description: string;
  benefits: string[];
}> = [
  {
    id: "creator",
    icon: "/role-selection/building-icon.png",
    title: "Create surveys & get insights",
    description:
      "Launch research projects and collect quality responses from engaged participants",
    benefits: [
      "Access to verified Stellar participants",
      "Advanced analytics and reporting",
      "Real-time response monitoring",
      "On-chain escrow for rewards",
    ],
  },
  {
    id: "hunter",
    icon: "/role-selection/cash-icon.png",
    title: "Take surveys & earn money",
    description:
      "Share your opinions with brands and get rewarded for your valuable feedback",
    benefits: [
      "Earn USDC for each completed mission",
      "Flexible schedule - work anytime",
      "Fair compensation for your time",
      "Instant payments via blockchain",
    ],
  },
];

const RoleSelection = () => {
  const [selectedRole, setSelectedRole] = useState<UserRole | null>(null);
  const router = useRouter();

  const handleContinue = () => {
    if (!selectedRole) return;

    saveUserRole(selectedRole);
    router.push(getDashboardRouteForRole(selectedRole));
  };

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#0b0a11] text-white selection:bg-purple-500/30">
      <div className="pointer-events-none absolute inset-0 z-0 flex items-center justify-center select-none">
        <svg
          viewBox="0 0 100 100"
          className="h-full w-full max-w-[56rem] fill-purple-700/20 blur-[140px]"
        >
          <path d="M50 0 C22.4 0 0 22.4 0 50 C0 77.6 22.4 100 50 100 C60 100 69 97 77 92 L85 100 L92 92 L84 84 C94 75 100 63 100 50 C100 22.4 77.6 0 50 0 Z M50 80 C33.4 80 20 66.6 20 50 C20 33.4 33.4 20 50 20 C66.6 20 80 33.4 80 50 C80 66.6 66.6 80 50 80 Z" />
        </svg>
      </div>

      <div className="relative z-10 mx-auto flex min-h-screen w-full max-w-6xl flex-col items-center justify-center px-5 py-8 font-inter sm:px-6 lg:px-8 lg:py-16">
        <div className="w-full rounded-[32px] border border-[#241B4A] bg-[#121015]/95 px-6 py-8 shadow-[0_24px_80px_rgba(0,0,0,0.45)] backdrop-blur sm:px-8 sm:py-10 lg:px-12 lg:py-14">
          <div className="flex flex-col items-center border-b border-[#241B4A] pb-8 text-center">
            <svg
              className="size-24"
              viewBox="0 0 85 49"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                fillRule="evenodd"
                clipRule="evenodd"
                d="M6.42928 37.6266L10.3292 43.2301H6.69637C2.99812 43.2301 0 40.2522 0 36.5793V12.4702H5.02228V34.9166C5.02228 36.0345 5.57772 37.0234 6.42928 37.6266Z"
                fill="#9011FF"
                className="fill-[#9011FF]"
              />
              <path
                fillRule="evenodd"
                clipRule="evenodd"
                d="M18.415 43.0206C21.3032 42.2826 23.4373 39.6782 23.4373 36.5793V6.65078C23.4373 2.97791 20.4392 0 16.7409 0H6.69637C2.99812 0 0 2.97791 0 6.65078V9.14483H5.02228V8.31348C5.02228 6.47704 6.52124 4.98809 8.37047 4.98809H15.0668C16.9161 4.98809 18.415 6.47704 18.415 8.31348V34.9166C18.415 36.753 16.9161 38.242 15.0668 38.242H10.928L18.415 49V43.0206Z"
                fill="#9011FF"
                className="fill-[#9011FF]"
              />
              <path
                d="M44.8858 16.4393H48.2072V42.8828H45.3679L45.2608 38.2538C43.9215 41.6058 41.0287 43.4148 37.0109 43.4148C31.9216 43.4148 29.2966 40.4353 29.2966 34.689V16.4393H32.618V34.2101C32.618 38.4666 34.3323 40.5417 37.868 40.5417C42.3144 40.5417 44.8858 37.5089 44.8858 32.3479V16.4393Z"
                fill="white"
                className="fill-white"
              />
              <path
                d="M55.3573 11.3847V5.58517H58.6786V11.3847H55.3573ZM55.3037 42.8828V16.4393H58.6251V42.8828H55.3037Z"
                fill="white"
                className="fill-white"
              />
              <path
                d="M85 42.8828H81.9465C81.625 42.0847 81.4643 40.4885 81.4107 38.3602C80.0179 41.6058 77.1251 43.4148 73.0537 43.4148C67.8037 43.4148 65.1787 40.4353 65.1787 34.7422V24.633C65.1787 18.8867 67.8037 15.9072 73.0537 15.9072C76.8036 15.9072 79.5893 17.4502 81.0893 20.2701V5.63838H84.4107V35.8595C84.4107 39.0519 84.5714 41.393 85 42.8828ZM73.9108 40.5417C78.4643 40.5417 81.0893 37.5089 81.0893 32.3479V26.9741C81.0893 21.8131 78.4643 18.7803 73.9108 18.7803C70.2144 18.7803 68.5001 20.8554 68.5001 25.1119V34.2101C68.5001 38.4666 70.2144 40.5417 73.9108 40.5417Z"
                fill="white"
                className="fill-white"
              />
            </svg>

            <div className="mt-8 flex max-w-2xl flex-col items-center gap-3">
              <h1 className="text-2xl font-semibold text-white sm:text-3xl lg:text-4xl">
                Account Type Selection
              </h1>
              <p className="text-sm text-[#B0B0B0] sm:text-base">
                Choose how you want to use Quid and continue into a dashboard that feels like the same product.
              </p>
            </div>
          </div>

          <div className="mt-8 grid gap-5 md:grid-cols-2">
            {ROLES.map((role) => (
              <button
                type="button"
                className={`group rounded-[24px] border p-5 text-left transition-all duration-200 ${
                  selectedRole === role.id
                    ? "border-[#9011FF] bg-[#1B1540] shadow-[0_0_0_1px_rgba(144,17,255,0.45),0_18px_45px_rgba(144,17,255,0.16)]"
                    : "border-[#241B4A] bg-[#141026] hover:border-[#9011FF]/70 hover:bg-[#18122D]"
                }`}
                key={role.id}
                onClick={() => setSelectedRole(role.id)}
              >
                <div className="flex items-start gap-3">
                  <div className="rounded-2xl border border-[#2A1F55] bg-[#0b0a11] p-2.5">
                    <Image
                      src={role.icon}
                      alt=""
                      width={40}
                      height={40}
                      className="size-10"
                    />
                  </div>
                  <div className="space-y-2">
                    <h3 className="text-lg font-medium text-white sm:text-xl">
                      {role.title}
                    </h3>
                    <p className="text-sm leading-6 text-[#B0B0B0]">
                      {role.description}
                    </p>
                  </div>
                </div>

                <div className="mt-6 rounded-[20px] border border-[#241B4A] bg-[#0f0c1a] p-4">
                  <p className="pb-2 text-sm font-medium text-[#B48CFF]">
                    Key Benefits
                  </p>
                  <ul className="space-y-2 text-sm leading-6 text-[#CFC9FF]">
                    {role.benefits.map((benefit) => (
                      <li className="ml-5 list-disc marker:text-[#9011FF]" key={benefit}>
                        {benefit}
                      </li>
                    ))}
                  </ul>
                </div>
              </button>
            ))}
          </div>

          <div className="mt-8 flex justify-center border-t border-[#241B4A] pt-8">
            <button
              type="button"
              className="flex min-w-[14rem] items-center justify-center rounded-xl bg-[#9011FF] px-10 py-3 font-semibold text-white shadow-lg shadow-purple-500/30 transition-all duration-200 hover:bg-purple-700 hover:shadow-purple-500/50 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={selectedRole === null}
              onClick={handleContinue}
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default RoleSelection;
