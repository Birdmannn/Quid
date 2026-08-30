export type UserRole = 'creator' | 'hunter';

/**
 * Issue #331: how the backend spells the same thing (Prisma `UserRole`).
 * 'hunter' is the product word for what the schema calls an EARNER.
 */
export type ServerUserRole = 'CREATOR' | 'EARNER';

export const USER_ROLE_STORAGE_KEY = 'quid_user_role';

export const ONBOARDING_ROUTES = {
  signUp: '/connect-wallet',
  accountType: '/account-type',
  creator: '/creator',
  hunter: '/hunter',
} as const;

export function saveUserRole(role: UserRole): void {
  if (typeof window === 'undefined') return;
  localStorage.setItem(USER_ROLE_STORAGE_KEY, role);
}

export function getUserRole(): UserRole | null {
  if (typeof window === 'undefined') return null;
  const role = localStorage.getItem(USER_ROLE_STORAGE_KEY);
  if (role === 'creator' || role === 'hunter') {
    return role;
  }
  return null;
}

/**
 * Issue #331: localStorage is now only a cache of the server role, so it has to
 * be clearable when the two disagree or the wallet disconnects.
 */
export function clearUserRole(): void {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(USER_ROLE_STORAGE_KEY);
}

/** Issue #331: UI role -> the value the API stores. */
export function toServerRole(role: UserRole): ServerUserRole {
  return role === 'creator' ? 'CREATOR' : 'EARNER';
}

/** Issue #331: API value -> UI role; anything unrecognised is "no role yet". */
export function fromServerRole(role: string | null | undefined): UserRole | null {
  if (role === 'CREATOR') return 'creator';
  if (role === 'EARNER') return 'hunter';
  return null;
}

export function getDashboardRouteForRole(role: UserRole): string {
  return role === 'creator'
    ? ONBOARDING_ROUTES.creator
    : ONBOARDING_ROUTES.hunter;
}
