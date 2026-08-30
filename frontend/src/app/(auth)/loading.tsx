export default function AuthLoading() {
  return (
    <div className="min-h-screen bg-[#0b0a11] text-white grid place-items-center overflow-x-hidden">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-[radial-gradient(75%_120%_at_50%_-5%,rgba(124,44,255,0.45)_0%,rgba(12,10,20,0.2)_55%,rgba(8,8,12,0.96)_100%)]" />
      </div>

      <div className="relative z-10 flex flex-col items-center px-4 pb-12 w-full max-w-sm">
        <div className="w-12 h-7 bg-white/20 animate-pulse rounded-md" />

        <div className="mt-20 flex flex-col items-center w-full">
          <div className="h-8 w-44 bg-white/20 animate-pulse rounded-md mb-2" />
          <div className="h-4 w-72 bg-white/10 animate-pulse rounded-md" />
        </div>

        <div className="flex flex-col gap-6 w-full border border-white/10 rounded-2xl pt-4 mt-10 overflow-hidden">
          <div className="flex flex-col gap-y-3 px-3">
            <div className="h-11 w-full bg-white/10 animate-pulse rounded-lg" />
          </div>
          <div className="bg-black/30 px-2 py-3">
            <div className="h-3 w-72 mx-auto bg-white/10 animate-pulse rounded-md" />
          </div>
        </div>

        <div className="mt-8 w-full border rounded-md border-pink-300/30 py-4 flex gap-2 items-center px-2">
          <div className="w-8 h-8 shrink-0 rounded-full bg-pink-300/30 animate-pulse" />
          <div className="flex-1 h-8 bg-white/10 animate-pulse rounded-md" />
        </div>
      </div>
    </div>
  );
}
