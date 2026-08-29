"use client";

import { useState } from "react";
import { uploadFeedbackToIpfs } from "@/lib/upload-api";
import { submitFeedbackToContract, SubmissionReceipt } from "@/lib/soroban-client";
import { useWallet } from "@/context/WalletProvider";
import {
  AlertCircle,
  CheckCircle2,
  Loader2,
  ShieldCheck,
  UploadCloud,
  X,
} from "lucide-react";

interface SubmitFeedbackModalProps {
  isOpen: boolean;
  onClose: () => void;
  quest: {
    id: string | number;
    title: string;
    brand: string;
    reward: string;
  };
  onSuccess?: (receipt: SubmissionReceipt) => void;
}

type SubmissionStep = "form" | "uploading_ipfs" | "signing_chain" | "success" | "error";

export default function SubmitFeedbackModal({
  isOpen,
  onClose,
  quest,
  onSuccess,
}: SubmitFeedbackModalProps) {
  const { connected, publicKey, connect } = useWallet();

  const [feedbackText, setFeedbackText] = useState("");
  const [proofUrl, setProofUrl] = useState("");
  const [sentiment, setSentiment] = useState(5);
  const [step, setStep] = useState<SubmissionStep>("form");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [receipt, setReceipt] = useState<SubmissionReceipt | null>(null);
  const [uploadedCid, setUploadedCid] = useState<string | null>(null);

  if (!isOpen) return null;

  const resetState = () => {
    setStep("form");
    setErrorMessage(null);
    setReceipt(null);
    setUploadedCid(null);
    onClose();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!feedbackText.trim()) {
      setErrorMessage("Please enter your detailed feedback before submitting.");
      return;
    }

    if (!connected || !publicKey) {
      try {
        await connect();
      } catch {
        setErrorMessage("Wallet connection required to submit on-chain.");
        return;
      }
    }

    setErrorMessage(null);

    // Step 1: Upload to IPFS via Backend API
    setStep("uploading_ipfs");
    let cid = "";
    try {
      // Map string IDs to numeric/u64 mission ID
      const numericMissionId =
        typeof quest.id === "number"
          ? quest.id
          : parseInt(quest.id.replace(/\D/g, "") || "1", 10);

      const uploadResult = await uploadFeedbackToIpfs({
        missionId: numericMissionId,
        hunterAddress: publicKey!,
        feedbackText,
        proofUrl,
        sentiment: sentiment / 5,
        metadata: {
          questTitle: quest.title,
          questBrand: quest.brand,
          reward: quest.reward,
        },
      });

      cid = uploadResult.cid;
      setUploadedCid(cid);
    } catch (uploadErr: unknown) {
      const msg = uploadErr instanceof Error ? uploadErr.message : "Failed to upload feedback to IPFS";
      setErrorMessage(`Upload Blocked: ${msg}. Smart contract was NOT invoked.`);
      setStep("error");
      return; // Upload failure BLOCKS chain call
    }

    // Step 2: Invoke submit_feedback on Soroban smart contract with CID
    setStep("signing_chain");
    try {
      const numericMissionId =
        typeof quest.id === "number"
          ? quest.id
          : parseInt(quest.id.replace(/\D/g, "") || "1", 10);

      const txReceipt = await submitFeedbackToContract({
        missionId: numericMissionId,
        hunterAddress: publicKey!,
        ipfsCid: cid,
        stakeAmount: BigInt(10000000), // 1 XLM refundable stake
      });

      setReceipt(txReceipt);
      setStep("success");
      onSuccess?.(txReceipt);
    } catch (chainErr: unknown) {
      const msg = chainErr instanceof Error ? chainErr.message : "Soroban transaction failed";
      setErrorMessage(`Smart Contract Error: ${msg}`);
      setStep("error");
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm">
      <div className="relative w-full max-w-xl rounded-2xl border border-white/15 bg-[#120F1D] p-6 text-white shadow-2xl md:p-8">
        {/* Header */}
        <div className="flex items-start justify-between border-b border-white/10 pb-4">
          <div>
            <span className="text-xs font-semibold uppercase tracking-wider text-[#B78CFF]">
              Submit Feedback & Proof
            </span>
            <h2 className="mt-1 text-xl font-bold text-white">{quest.title}</h2>
            <p className="text-sm text-white/60">
              Reward: <span className="font-semibold text-[#B78CFF]">{quest.reward}</span> • {quest.brand}
            </p>
          </div>
          <button
            onClick={resetState}
            className="rounded-lg p-1 text-white/50 hover:bg-white/10 hover:text-white"
          >
            <X className="size-5" />
          </button>
        </div>

        {/* Modal Body */}
        <div className="mt-5">
          {step === "form" && (
            <form onSubmit={handleSubmit} className="space-y-5">
              {/* Feedback text area */}
              <div>
                <label className="block text-sm font-medium text-white/90">
                  Feedback & Findings <span className="text-red-400">*</span>
                </label>
                <p className="mb-2 text-xs text-white/50">
                  Provide constructive, honest review points, bug reports, and UX suggestions.
                </p>
                <textarea
                  rows={4}
                  required
                  value={feedbackText}
                  onChange={(e) => setFeedbackText(e.target.value)}
                  placeholder="Detailed observations, test steps performed, bugs discovered, or dApp feedback..."
                  className="w-full rounded-xl border border-white/15 bg-white/[0.04] p-3 text-sm text-white placeholder-white/30 focus:border-[#B78CFF] focus:outline-none"
                />
              </div>

              {/* Proof URL */}
              <div>
                <label className="block text-sm font-medium text-white/90">
                  Proof / Screenshots URL (Optional)
                </label>
                <input
                  type="url"
                  value={proofUrl}
                  onChange={(e) => setProofUrl(e.target.value)}
                  placeholder="https://github.com/... or Loom / screenshot URL"
                  className="mt-1 w-full rounded-xl border border-white/15 bg-white/[0.04] p-3 text-sm text-white placeholder-white/30 focus:border-[#B78CFF] focus:outline-none"
                />
              </div>

              {/* Sentiment / Rating */}
              <div>
                <label className="block text-sm font-medium text-white/90">
                  Overall Rating: {sentiment} / 5
                </label>
                <input
                  type="range"
                  min="1"
                  max="5"
                  value={sentiment}
                  onChange={(e) => setSentiment(parseInt(e.target.value, 10))}
                  className="mt-2 w-full accent-[#B78CFF]"
                />
              </div>

              {/* Stake UX Explanation */}
              <div className="flex gap-3 rounded-xl border border-[#B78CFF]/30 bg-[#B78CFF]/10 p-4 text-sm text-white/85">
                <ShieldCheck className="size-5 shrink-0 text-[#B78CFF]" />
                <div>
                  <p className="font-semibold text-white">Refundable Anti-Spam Stake (1 XLM)</p>
                  <p className="mt-1 text-xs text-white/70">
                    To maintain high feedback quality, submitting feedback requires a 1 XLM stake. This stake is locked safely in the Soroban smart contract and is <strong>fully refunded</strong> directly to your wallet when the creator approves your submission.
                  </p>
                </div>
              </div>

              {errorMessage && (
                <div className="flex items-center gap-2 rounded-lg border border-red-500/30 bg-red-500/10 p-3 text-xs text-red-300">
                  <AlertCircle className="size-4 shrink-0 text-red-400" />
                  <span>{errorMessage}</span>
                </div>
              )}

              {/* Submit CTA */}
              <div className="flex items-center justify-end gap-3 pt-2">
                <button
                  type="button"
                  onClick={resetState}
                  className="rounded-xl px-4 py-2.5 text-sm font-medium text-white/70 hover:bg-white/10 hover:text-white"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="inline-flex items-center gap-2 rounded-xl bg-[linear-gradient(135deg,#9011FF_0%,#B78CFF_100%)] px-6 py-2.5 text-sm font-semibold text-white shadow-lg transition-transform hover:opacity-95 active:scale-[0.98]"
                >
                  <UploadCloud className="size-4" />
                  Upload & Submit On-Chain
                </button>
              </div>
            </form>
          )}

          {step === "uploading_ipfs" && (
            <div className="py-10 text-center space-y-4">
              <Loader2 className="mx-auto size-10 animate-spin text-[#B78CFF]" />
              <h3 className="text-lg font-semibold">Step 1: Uploading to IPFS...</h3>
              <p className="max-w-md mx-auto text-sm text-white/60">
                Packaging feedback and pinning proof to backend IPFS node to generate content identifier (CID).
              </p>
            </div>
          )}

          {step === "signing_chain" && (
            <div className="py-10 text-center space-y-4">
              <Loader2 className="mx-auto size-10 animate-spin text-[#B78CFF]" />
              <h3 className="text-lg font-semibold">Step 2: Submitting to Soroban Contract...</h3>
              <p className="max-w-md mx-auto text-sm text-white/60">
                Please approve the transaction in your Freighter wallet. CID <code className="text-[#B78CFF]">{uploadedCid?.slice(0, 16)}...</code> will be recorded on-chain.
              </p>
            </div>
          )}

          {step === "success" && receipt && (
            <div className="py-6 text-center space-y-4">
              <CheckCircle2 className="mx-auto size-14 text-emerald-400" />
              <h3 className="text-xl font-bold text-white">Feedback Submitted Successfully!</h3>
              <p className="text-sm text-white/75">
                Your proof has been pinned to IPFS and submitted to the <code className="text-[#B78CFF]">quid-store</code> smart contract.
              </p>

              <div className="rounded-xl border border-white/10 bg-white/[0.03] p-4 text-left text-xs space-y-2">
                <div className="flex justify-between">
                  <span className="text-white/50">IPFS CID:</span>
                  <span className="font-mono text-[#B78CFF]">{receipt.cid}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-white/50">Transaction:</span>
                  <span className="font-mono text-white/80">{receipt.txHash.slice(0, 18)}...</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-white/50">Stake Status:</span>
                  <span className="text-emerald-400 font-medium">1 XLM Locked (Refundable)</span>
                </div>
              </div>

              <div className="pt-3">
                <button
                  type="button"
                  onClick={resetState}
                  className="rounded-xl bg-[linear-gradient(135deg,#9011FF_0%,#B78CFF_100%)] px-8 py-2.5 text-sm font-semibold text-white hover:opacity-95"
                >
                  Done
                </button>
              </div>
            </div>
          )}

          {step === "error" && (
            <div className="py-6 text-center space-y-4">
              <AlertCircle className="mx-auto size-12 text-red-400" />
              <h3 className="text-lg font-bold text-white">Submission Failed</h3>
              <p className="text-sm text-red-300 max-w-md mx-auto">{errorMessage}</p>

              <div className="flex justify-center gap-3 pt-3">
                <button
                  type="button"
                  onClick={() => setStep("form")}
                  className="rounded-xl border border-white/20 px-5 py-2 text-sm font-medium text-white hover:bg-white/10"
                >
                  Try Again
                </button>
                <button
                  type="button"
                  onClick={resetState}
                  className="rounded-xl px-5 py-2 text-sm text-white/60 hover:text-white"
                >
                  Close
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
