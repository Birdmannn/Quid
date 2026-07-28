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
    return null;
  }

  return <>{children}</>;
}
