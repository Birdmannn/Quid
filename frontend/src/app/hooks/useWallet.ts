"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  useWallet as useWalletContext,
  type Balance,
} from "@/context/WalletProvider";

/**
 * Horizon endpoint. Defaults to the public testnet so unfunded accounts can be
 * funded via Friendbot. Override with NEXT_PUBLIC_HORIZON_URL for other networks.
 */
export const HORIZON_URL =
  process.env.NEXT_PUBLIC_HORIZON_URL ?? "https://horizon-testnet.stellar.org";

export const FRIENDBOT_URL =
  process.env.NEXT_PUBLIC_FRIENDBOT_URL ?? "https://friendbot.stellar.org";

/** A single balance line item, normalized from the wallet context. */
export interface WalletBalance {
  assetType: string;
  assetCode: string;
  assetIssuer?: string;
  balance: string;
}

/** A normalized payment, derived from Horizon's `/accounts/{id}/payments`. */
export interface WalletTransaction {
  id: string;
  hash: string;
  type: string;
  direction: "in" | "out";
  counterparty: string;
  amount: string;
  assetCode: string;
  date: string;
  successful: boolean;
}

interface HorizonPayment {
  id: string;
  type: string;
  transaction_hash: string;
  transaction_successful?: boolean;
  created_at: string;
  amount?: string;
  asset_type?: string;
  asset_code?: string;
  from?: string;
  to?: string;
  funder?: string;
  account?: string;
  starting_balance?: string;
}

export interface UseWalletResult {
  /** Connected account public key (empty string when not connected). */
  publicKey: string;
  isConnected: boolean;
  /** All balances on the account, native first. */
  balances: WalletBalance[];
  /** Native XLM balance, or null when the account is unfunded. */
  xlm: string | null;
  /** USDC balance, or null when the account holds no USDC trustline. */
  usdc: string | null;
  /** Recent payments to/from the account, newest first. */
  transactions: WalletTransaction[];
  /** False when Horizon has no record of the account yet (needs funding). */
  funded: boolean;
  loading: boolean;
  error: string | null;
  /** Re-fetch balances + transactions from Horizon without a page reload. */
  refreshBalances: () => void;
  /** Fund the account on testnet via Friendbot, then refresh. */
  fundWithFriendbot: () => Promise<void>;
  funding: boolean;
}

function parseBalances(raw: Balance[]): WalletBalance[] {
  return raw
    .map((b) => ({
      assetType: b.asset_type,
      assetCode: b.asset_type === "native" ? "XLM" : b.asset_code ?? "",
      assetIssuer: b.asset_issuer,
      balance: b.balance,
    }))
    // Native asset first, then the rest in the context's order.
    .sort((a, b) =>
      a.assetType === "native" ? -1 : b.assetType === "native" ? 1 : 0,
    );
}

function parsePayments(
  records: HorizonPayment[],
  pk: string,
): WalletTransaction[] {
  const out: WalletTransaction[] = [];

  for (const r of records) {
    let amount: string | undefined;
    let assetCode = "XLM";
    let direction: "in" | "out";
    let counterparty: string | undefined;

    if (r.type === "create_account") {
      amount = r.starting_balance;
      direction = r.account === pk ? "in" : "out";
      counterparty = direction === "in" ? r.funder : r.account;
    } else if (
      r.type === "payment" ||
      r.type === "path_payment_strict_send" ||
      r.type === "path_payment_strict_receive"
    ) {
      amount = r.amount;
      assetCode = r.asset_type === "native" ? "XLM" : r.asset_code ?? "";
      direction = r.to === pk ? "in" : "out";
      counterparty = direction === "in" ? r.from : r.to;
    } else {
      // Skip operation types we can't represent as a simple debit/credit.
      continue;
    }

    out.push({
      id: r.id,
      hash: r.transaction_hash,
      type: r.type,
      direction,
      counterparty: counterparty ?? "",
      amount: amount ?? "0",
      assetCode,
      date: r.created_at,
      successful: r.transaction_successful !== false,
    });
  }

  return out;
}

/**
 * Wallet view-model for the creator wallet page. Balances come from the shared
 * WalletProvider (single source of truth); this hook adds the pieces the page
 * needs on top — normalized balances, real transaction history and an unfunded
 * flag from Horizon's payments endpoint, plus Friendbot funding and refresh.
 */
export function useWallet(): UseWalletResult {
  const {
    publicKey: contextPublicKey,
    connected,
    balances: contextBalances,
    refreshBalances: refreshContextBalances,
  } = useWalletContext();

  const publicKey = contextPublicKey ?? "";
  const isConnected = connected;

  const [transactions, setTransactions] = useState<WalletTransaction[]>([]);
  const [funded, setFunded] = useState<boolean>(true);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [funding, setFunding] = useState<boolean>(false);

  const balances = useMemo(
    () => parseBalances(contextBalances),
    [contextBalances],
  );

  // Transaction history + funded status come from the account's payments
  // endpoint, which 404s for accounts that have not been created on the ledger.
  const fetchActivity = useCallback(async () => {
    if (!publicKey) {
      setTransactions([]);
      setFunded(true);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await fetch(
        `${HORIZON_URL}/accounts/${publicKey}/payments?order=desc&limit=15`,
      );

      if (res.status === 404) {
        setTransactions([]);
        setFunded(false);
        return;
      }

      if (!res.ok) {
        throw new Error(`Horizon responded with ${res.status}`);
      }

      const data = await res.json();
      const records: HorizonPayment[] = data?._embedded?.records ?? [];
      setTransactions(parsePayments(records, publicKey));
      setFunded(true);
    } catch (err) {
      console.error("Failed to load wallet activity:", err);
      setError("Unable to load wallet activity. Check your connection and retry.");
    } finally {
      setLoading(false);
    }
  }, [publicKey]);

  useEffect(() => {
    // Defer to a microtask so the loading/reset setState calls don't run
    // synchronously inside the effect (avoids cascading renders).
    let active = true;
    void Promise.resolve().then(() => {
      if (active) void fetchActivity();
    });
    return () => {
      active = false;
    };
  }, [fetchActivity]);

  const refreshBalances = useCallback(async () => {
    await Promise.all([refreshContextBalances(), fetchActivity()]);
  }, [refreshContextBalances, fetchActivity]);

  const fundWithFriendbot = useCallback(async () => {
    if (!publicKey) return;
    setFunding(true);
    setError(null);
    try {
      const res = await fetch(`${FRIENDBOT_URL}/?addr=${publicKey}`);
      if (!res.ok) {
        // Friendbot replies 400 "account already funded to starting balance"
        // for accounts that already exist — surface that clearly.
        let detail = `Friendbot responded with ${res.status}`;
        try {
          const body = await res.json();
          if (typeof body?.detail === "string") detail = body.detail;
        } catch {
          // non-JSON body; keep the status-based message
        }
        if (detail.toLowerCase().includes("already funded")) {
          throw new Error(
            "This account is already funded. Friendbot only funds new testnet accounts.",
          );
        }
        throw new Error(detail);
      }
      await refreshBalances();
    } catch (err) {
      console.error("Friendbot funding failed:", err);
      setError(
        err instanceof Error
          ? err.message
          : "Friendbot funding failed. Try again in a moment.",
      );
    } finally {
      setFunding(false);
    }
  }, [publicKey, refreshBalances]);

  const find = (code: string) =>
    balances.find((b) => b.assetCode === code)?.balance ?? null;

  return {
    publicKey,
    isConnected,
    balances,
    xlm: find("XLM"),
    usdc: find("USDC"),
    transactions,
    funded,
    loading,
    error,
    refreshBalances,
    fundWithFriendbot,
    funding,
  };
}
