import Link from "next/link";

export default function NotFound() {
  return (
    <html lang="en">
      <body className="bg-[#0D0B10] text-white antialiased">
        <div className="flex min-h-screen items-center justify-center px-4">
          <div className="mx-auto max-w-md text-center">
            <div className="mb-8">
              <p className="mb-2 text-[10rem] font-bold leading-none tracking-tighter text-[#9011FF] sm:text-[12rem]">
                404
              </p>
              <div className="mb-6 h-px w-24 mx-auto bg-gradient-to-r from-transparent via-[#9011FF] to-transparent" />
            </div>

            <h1 className="mb-3 text-2xl font-semibold sm:text-3xl">
              Page not found
            </h1>
            <p className="mb-8 text-base text-[#8C86B8] sm:text-lg">
              The page you&apos;re looking for doesn&apos;t exist or has been moved.
            </p>

            <div className="flex flex-col items-center justify-center gap-3 sm:flex-row sm:gap-4">
              <Link
                href="/"
                className="inline-flex items-center justify-center rounded-lg bg-[#9011FF] px-6 py-3 text-sm font-medium text-white transition-all hover:bg-[#7a0edb] focus:outline-none focus:ring-2 focus:ring-[#9011FF] focus:ring-offset-2 focus:ring-offset-[#0D0B10]"
              >
                Back to home
              </Link>
              <Link
                href="/missions"
                className="inline-flex items-center justify-center rounded-lg border border-white/10 bg-white/[0.03] px-6 py-3 text-sm font-medium text-white transition-all hover:border-white/20 hover:bg-white/[0.06] focus:outline-none focus:ring-2 focus:ring-[#9011FF] focus:ring-offset-2 focus:ring-offset-[#0D0B10]"
              >
                Browse missions
              </Link>
            </div>
          </div>
        </div>
      </body>
    </html>
  );
}
