'use client';

import { useEffect, useState } from 'react';
import { useWallet } from '@/context/WalletProvider';
import { getUserRole, saveUserRole, type UserRole } from '@/lib/onboarding';
import { fetchServerUserRole } from '@/lib/user-api';

export type RoleSource = 'server' | 'local' | 'none';

export interface UserRoleState {
  role: UserRole | null;
  source: RoleSource;
  /** True until the server has been consulted (or ruled out). */
  loading: boolean;
}

/**
 * Issue #331: one place that answers "what is this user's role?".
 *
 * localStorage is read first so the dashboard can paint immediately, then the
 * server is consulted and wins if it has an answer - that is what keeps two
 * devices consistent. The server value is mirrored back into localStorage so
 * the next visit starts from the right place.
 */
export function useUserRole(): UserRoleState {
  const { publicKey } = useWallet();
  const [state, setState] = useState<UserRoleState>(() => ({
    role: null,
    source: 'none',
    loading: true,
  }));

  useEffect(() => {
    let active = true;
    const address = publicKey;

    // Resolved off the render path: the eslint react-hooks rules (rightly)
    // reject a synchronous setState inside an effect body, and `loading` is
    // what the guard shows a spinner for while this settles.
    async function resolveRole(): Promise<UserRoleState> {
      const localRole = getUserRole();

      if (!address) {
        return {
          role: localRole,
          source: localRole ? 'local' : 'none',
          loading: false,
        };
      }

      let serverRole: UserRole | null = null;
      try {
        serverRole = await fetchServerUserRole(address);
      } catch {
        // An unreachable API must not lock a user out of their dashboard.
        serverRole = null;
      }

      if (serverRole) {
        // The server is the source of truth; mirror it so the next visit
        // starts from the right place even before the network answers.
        saveUserRole(serverRole);
        return { role: serverRole, source: 'server', loading: false };
      }

      return {
        role: localRole,
        source: localRole ? 'local' : 'none',
        loading: false,
      };
    }

    void resolveRole().then((next) => {
      if (active) setState(next);
    });

    return () => {
      active = false;
    };
  }, [publicKey]);

  return state;
}
