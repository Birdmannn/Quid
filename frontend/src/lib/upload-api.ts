import { signFreighterTransaction } from "@/lib/freighter-wallet";
import type { Networks } from "@stellar/stellar-sdk";

const API_URL = process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") || "http://localhost:3001";
const SESSION_KEY = "quid_hunter_auth";

export interface FeedbackPayload {
  missionId: string | number;
  hunterAddress: string;
  feedbackText: string;
  proofUrl?: string;
  sentiment?: number;
  metadata?: Record<string, unknown>;
}

export interface UploadResponse {
  cid: string;
  filename?: string;
  size?: number;
  [key: string]: unknown;
}

interface ChallengeResponse {
  transaction: string;
  networkPassphrase: Networks;
}

interface VerifyResponse {
  access_token: string;
}

function readStoredSession(address: string): string | null {
  try {
    const stored = localStorage.getItem(`${SESSION_KEY}_${address}`);
    if (!stored) return null;
    const parsed = JSON.parse(stored) as { accessToken: string; expiresAt?: number };
    return parsed.accessToken || null;
  } catch {
    return null;
  }
}

function storeSession(address: string, token: string): void {
  try {
    localStorage.setItem(
      `${SESSION_KEY}_${address}`,
      JSON.stringify({ accessToken: token, timestamp: Date.now() }),
    );
  } catch (e) {
    console.warn("Failed to cache auth token:", e);
  }
}

export async function authenticateHunter(address: string): Promise<string> {
  const cached = readStoredSession(address);
  if (cached) return cached;

  const challengeRes = await fetch(
    `${API_URL}/auth/challenge?address=${encodeURIComponent(address)}`,
  );
  if (!challengeRes.ok) {
    throw new Error(`Failed to request wallet auth challenge: ${challengeRes.statusText}`);
  }

  const challenge = (await challengeRes.json()) as ChallengeResponse;
  const signedXdr = await signFreighterTransaction(
    challenge.transaction,
    address,
    challenge.networkPassphrase,
  );

  const verifyRes = await fetch(`${API_URL}/auth/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ signedXdr }),
  });

  if (!verifyRes.ok) {
    throw new Error(`Wallet authentication verification failed: ${verifyRes.statusText}`);
  }

  const verified = (await verifyRes.json()) as VerifyResponse;
  if (!verified.access_token) {
    throw new Error("Auth verify response missing access token");
  }

  storeSession(address, verified.access_token);
  return verified.access_token;
}

/**
 * Uploads structured feedback JSON to backend IPFS endpoint and returns the CID.
 * Fails loudly so caller can prevent on-chain transaction if upload fails.
 */
export async function uploadFeedbackToIpfs(
  payload: FeedbackPayload,
): Promise<{ cid: string; data: UploadResponse }> {
  try {
    let token = readStoredSession(payload.hunterAddress);

    const doUpload = async (authToken?: string) => {
      const headers: Record<string, string> = {
        "Content-Type": "application/json",
      };
      if (authToken) {
        headers["Authorization"] = `Bearer ${authToken}`;
      }

      return fetch(`${API_URL}/upload/json`, {
        method: "POST",
        headers,
        body: JSON.stringify({
          ...payload,
          timestamp: new Date().toISOString(),
        }),
      });
    };

    let response = await doUpload(token ?? undefined);

    // If unauthorized, re-authenticate and retry once
    if (response.status === 401) {
      token = await authenticateHunter(payload.hunterAddress);
      response = await doUpload(token);
    }

    if (!response.ok) {
      const errorText = await response.text().catch(() => "");
      throw new Error(
        `Backend IPFS upload failed with status ${response.status}: ${errorText || response.statusText}`,
      );
    }

    const data = (await response.json()) as UploadResponse;

    if (!data.cid) {
      throw new Error("Upload response did not return a valid IPFS CID");
    }

    return { cid: data.cid, data };
  } catch (error: unknown) {
    const msg = error instanceof Error ? error.message : "Unknown IPFS upload error";
    console.error("IPFS Upload Error:", error);
    throw new Error(`IPFS Upload Failed: ${msg}. On-chain submission aborted.`);
  }
}
