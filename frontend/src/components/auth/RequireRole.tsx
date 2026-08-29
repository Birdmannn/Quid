'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useUserRole } from '@/app/hooks/useUserRole';
import {
  ONBOARDING_ROUTES,
  getDashboardRouteForRole,
  type UserRole,
} from '@/lib/onboarding';

/**
 * Issue #331: creator/hunter routes respect the role.
 *
 * The server role wins when it is available (see `useUserRole`); a user who
 * picked "hunter" on another device is sent to the hunter dashboard here even
 * if this browser's localStorage says otherwise. A user with no role anywhere
 * goes back to account selection instead of seeing an empty dashboard.
 */
export default function RequireRole({
  role: requiredRole,
  children,
}: {
  role: UserRole;
  children: React.ReactNode;
}) {
  const { role, loading } = useUserRole();
  const router = useRouter();

  useEffect(() => {
    if (loading) return;

    if (!role) {
      router.replace(ONBOARDING_ROUTES.accountType);
      return;
    }

    if (role !== requiredRole) {
      router.replace(getDashboardRouteForRole(role));
    }
  }, [loading, role, requiredRole, router]);

  if (loading || !role || role !== requiredRole) {
    return (
      <div className="flex h-screen items-center justify-center bg-[#0D0B10] text-white">
        <div className="text-center">
          <div className="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2 border-[#9011FF] border-t-transparent" />
          <p className="text-sm text-[#8C86B8]">Checking your account type…</p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
