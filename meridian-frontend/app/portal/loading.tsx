export default function PortalLoading() {
  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <div className="relative">
          <div className="w-12 h-12 rounded-full border-2 border-emerald-500/20 border-t-emerald-500 animate-spin mx-auto" />
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="w-5 h-5 rounded-full bg-emerald-500/10 border border-emerald-500/30" />
          </div>
        </div>
        <p className="mt-4 font-mono text-xs uppercase tracking-wider text-gray-500">
          Loading...
        </p>
      </div>
    </div>
  );
}
