export type UserRole = 'creator' | 'hunter';

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

export function getDashboardRouteForRole(role: UserRole): string {
  return role === 'creator'
    ? ONBOARDING_ROUTES.creator
    : ONBOARDING_ROUTES.hunter;
}
