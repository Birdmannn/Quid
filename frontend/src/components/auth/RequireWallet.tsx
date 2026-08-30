"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@/context/WalletProvider";
import { ONBOARDING_ROUTES } from "@/lib/onboarding";

export default function RequireWallet({
  children,
}: {
  children: React.ReactNode;
}) {
  const { connected } = useWallet();
  const router = useRouter();

  useEffect(() => {
    if (!connected) {
      router.replace(ONBOARDING_ROUTES.signUp);
    }
  }, [connected, router]);

  if (!connected) {
    return (
      <div className="flex h-screen items-center justify-center bg-[#0D0B10] text-white">
        <div className="text-center">
          <div className="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2 border-[#9011FF] border-t-transparent" />
          <p className="text-sm text-[#8C86B8]">Checking wallet connection…</p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
