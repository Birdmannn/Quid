'use client';

import Image from 'next/image';
import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Check, OctagonAlert } from 'lucide-react';
import type { SupportedWallet } from '@/context/WalletProvider';
import { useWallet } from '@/context/WalletProvider';
import { ONBOARDING_ROUTES } from '@/lib/onboarding';

export default function ConnectWalletPage() {
  const { connect, connected, publicKey, walletName, getAvailableWallets } =
    useWallet();
  const [availableWallets, setAvailableWallets] = useState<SupportedWallet[]>([]);
  const [isConnecting, setIsConnecting] = useState(false);
  const router = useRouter();

  useEffect(() => {
    const listAvailableWallets = async () => {
      const list = await getAvailableWallets();
      setAvailableWallets(list);
    };

    void listAvailableWallets();
  }, [getAvailableWallets]);

  const handleWalletSelect = async () => {
    if (connected) {
      router.push(ONBOARDING_ROUTES.accountType);
      return;
    }

    setIsConnecting(true);
    try {
      const address = await connect();
      if (address) {
        router.push(ONBOARDING_ROUTES.accountType);
      }
    } catch (error) {
      console.error('Wallet connection failed:', error);
    } finally {
      setIsConnecting(false);
    }
  };

  const shortAddress = publicKey
    ? `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}`
    : '';

  return (
    <div className="min-h-screen bg-[#0b0a11] text-white grid place-items-center overflow-x-hidden">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-[radial-gradient(75%_120%_at_50%_-5%,rgba(124,44,255,0.45)_0%,rgba(12,10,20,0.2)_55%,rgba(8,8,12,0.96)_100%)]" />
      </div>

      <div className="relative z-10 flex flex-col items-center px-4 pb-12 w-full max-w-sm">
        <Image
          src="/Quid Logo.png"
          alt="Quid Logo"
          width={28}
          height={28}
          className="h-7 w-12 mr-2"
          priority
        />

        <div className="flex flex-col items-center mt-20">
          <p className="text-3xl text-white">Connect a wallet</p>
          <p className="text-sm text-gray-500 text-center max-w-sm mt-2">
            Choose a wallet to continue signing up. More options will be added
            soon.
          </p>
          {connected && (
            <p className="mt-3 text-sm text-green-400">
              Connected to {walletName} ({shortAddress})
            </p>
          )}
        </div>

        <div className="flex flex-col gap-6 w-full bg-transparent backdrop-blur-md border border-white/10 shadow-xl rounded-2xl pt-4 mt-10">
          <div className="flex flex-col gap-y-3">
            {availableWallets.map((wallet) => {
              const isActive = connected && walletName === wallet.name;

              return (
                <button
                  key={wallet.id}
                  type="button"
                  disabled={isConnecting}
                  className={`flex px-3 py-2.5 min-h-11 items-center rounded-lg backdrop-blur-md border shadow-xl justify-between w-full transition-colors disabled:opacity-60 ${
                    isActive
                      ? 'bg-[#9011FF]/30 border-[#9011FF]/50'
                      : 'bg-white/20 border-white/10 hover:bg-white/30'
                  }`}
                  onClick={handleWalletSelect}
                >
                  <div className="flex gap-x-4 items-center">
                    <Image
                      src={wallet.icon}
                      alt={wallet.name}
                      width={25}
                      height={25}
                      unoptimized
                    />
                    <p>{wallet.name}</p>
                  </div>
                  <span className="text-xs text-white/60 self-center flex items-center gap-1">
                    {isActive ? (
                      <>
                        <Check className="h-3.5 w-3.5 text-green-400" />
                        Connected
                      </>
                    ) : isConnecting ? (
                      'Connecting...'
                    ) : (
                      'Connect →'
                    )}
                  </span>
                </button>
              );
            })}
          </div>

          <div className="bg-black/65 rounded-b-2xl px-2 py-3">
            <p className="text-[11px] text-center text-gray-600">
              By connecting your wallet, you agree to our{' '}
              <span className="font-bold text-gray-200">Terms and Conditions</span>{' '}
              and{' '}
              <span className="font-bold text-gray-200">Privacy Policy</span>
            </p>
          </div>
        </div>

        {connected && (
          <button
            type="button"
            onClick={() => router.push(ONBOARDING_ROUTES.accountType)}
            className="mt-8 bg-[#9011FF] hover:bg-purple-700 text-white font-semibold rounded-xl px-10 py-3 transition-colors"
          >
            Continue to account selection
          </button>
        )}

        <div className="mt-8 w-full border rounded-md border-pink-300 mx-auto py-4 flex gap-2 items-center px-2">
          <OctagonAlert className="text-2xl text-pink-300 shrink-0" />
          <p className="text-[10px]">
            Quid will never ask for your private keys or seed phrases. Only
            connect wallets you trust and control.
          </p>
        </div>
      </div>
    </div>
  );
}
