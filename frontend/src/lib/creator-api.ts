import { signFreighterTransaction } from "@/lib/freighter-wallet";
import type { Networks } from "@stellar/stellar-sdk";

const API_URL = process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "");
const SESSION_KEY = "quid_creator_auth";

interface CreatorSession {
  address: string;
  accessToken: string;
}

interface ChallengeResponse {
  transaction: string;
  networkPassphrase: Networks;
}

interface VerifyResponse {
  access_token: string;
}

function getApiUrl(path: string): string {
  if (!API_URL) {
    throw new Error("NEXT_PUBLIC_API_URL is not configured");
  }

  return `${API_URL}${path.startsWith("/") ? path : `/${path}`}`;
}

/**
 * Issue #331: whether an API base URL is configured at all. A local frontend
 * running without a backend still has to get through onboarding, so callers
 * use this to fall back to the localStorage-only behaviour instead of throwing.
 */
export function isApiConfigured(): boolean {
  return Boolean(API_URL);
}

/**
 * Issue #331: whether a SEP-10 session already exists for this address.
 *
 * Guards use it to decide whether the server role is reachable *without*
 * popping a wallet signature prompt on every dashboard mount.
 */
export function hasApiSession(address: string): boolean {
  if (typeof window === 'undefined') return false;
  return readSession(address) !== null;
}

function readSession(address: string): CreatorSession | null {
  try {
    const stored = localStorage.getItem(SESSION_KEY);
    if (!stored) return null;

    const session = JSON.parse(stored) as Partial<CreatorSession>;
    if (session.address !== address || !session.accessToken) {
      localStorage.removeItem(SESSION_KEY);
      return null;
    }

    return session as CreatorSession;
  } catch {
    localStorage.removeItem(SESSION_KEY);
    return null;
  }
}

function clearSession(): void {
  localStorage.removeItem(SESSION_KEY);
}

async function authenticate(address: string): Promise<CreatorSession> {
  const challengeResponse = await fetch(
    getApiUrl(`/auth/challenge?address=${encodeURIComponent(address)}`),
  );
  if (!challengeResponse.ok) {
    throw new Error("Unable to start wallet authentication");
  }

  const challenge = (await challengeResponse.json()) as ChallengeResponse;
  const signedXdr = await signFreighterTransaction(
    challenge.transaction,
    address,
    challenge.networkPassphrase,
  );

  const verifyResponse = await fetch(getApiUrl("/auth/verify"), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ signedXdr }),
  });
  if (!verifyResponse.ok) {
    throw new Error("Wallet authentication failed");
  }

  const verified = (await verifyResponse.json()) as VerifyResponse;
  if (!verified.access_token) {
    throw new Error("Authentication response did not include an access token");
  }

  const session = { address, accessToken: verified.access_token };
  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  return session;
}

async function getSession(address: string): Promise<CreatorSession> {
  return readSession(address) ?? authenticate(address);
}

export async function creatorApiFetch(
  path: string,
  address: string,
  init: RequestInit = {},
): Promise<Response> {
  const request = async (accessToken: string) => {
    const headers = new Headers(init.headers);
    headers.set("Authorization", `Bearer ${accessToken}`);

    return fetch(getApiUrl(path), { ...init, headers });
  };

  let session = await getSession(address);
  let response = await request(session.accessToken);

  if (response.status === 401) {
    clearSession();
    session = await authenticate(address);
    response = await request(session.accessToken);
  }

  return response;
}
