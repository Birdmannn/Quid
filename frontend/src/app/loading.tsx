export default function Loading() {
  return (
    <div className="min-h-screen bg-[#0b0a11] text-white">
      <div className="pointer-events-none fixed inset-0 z-0">
        <div className="absolute inset-0 bg-[radial-gradient(75%_120%_at_50%_-5%,rgba(124,44,255,0.45)_0%,rgba(12,10,20,0.2)_55%,rgba(8,8,12,0.96)_100%)]" />
      </div>
      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
        <div className="mb-12 flex items-center justify-between">
          <div className="w-12 h-7 bg-white/20 animate-pulse rounded-md" />
          <div className="flex gap-4">
            <div className="h-8 w-20 bg-white/10 animate-pulse rounded-md" />
            <div className="h-8 w-24 bg-white/10 animate-pulse rounded-md" />
          </div>
        </div>

        <div className="mb-16 mx-auto max-w-4xl">
          <div className="h-12 w-96 bg-white/20 animate-pulse rounded-md mb-4" />
          <div className="h-5 w-72 bg-white/10 animate-pulse rounded-md" />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-16">
          {[1, 2, 3].map((i) => (
            <div
              key={i}
              className="rounded-lg border border-white/5 bg-white/[0.03] p-8 animate-pulse"
            >
              <div className="h-4 w-24 bg-white/10 rounded-md mb-4" />
              <div className="h-10 w-32 bg-white/20 rounded-md" />
            </div>
          ))}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="rounded-lg border border-white/5 bg-white/[0.03] p-8 space-y-4 animate-pulse">
            <div className="h-8 w-40 bg-white/20 rounded-md mb-6" />
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-20 bg-white/10 rounded-lg" />
            ))}
          </div>
          <div className="rounded-lg border border-white/5 bg-white/[0.03] p-8 space-y-4 animate-pulse">
            <div className="h-8 w-40 bg-white/20 rounded-md mb-6" />
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-20 bg-white/10 rounded-lg" />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
