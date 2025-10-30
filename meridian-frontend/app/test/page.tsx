'use client';

export default function TestPage() {
  return (
    <div className="min-h-screen p-8">
      <h1 className="text-4xl font-bold mb-8">Style Test Page</h1>
      
      {/* Test 1: Standard Tailwind colors */}
      <div className="mb-8 p-4 bg-red-500 text-white rounded">
        Test 1: Standard Tailwind (red-500) - Should work
      </div>
      
      {/* Test 2: CSS variable colors */}
      <div className="mb-8 p-4 bg-background border-2 border-foreground rounded">
        Test 2: bg-background (CSS var) - Should be white/black based on theme
      </div>
      
      {/* Test 3: Emerald accent */}
      <div className="mb-8 p-4 bg-emerald-500 text-white rounded">
        Test 3: Emerald accent - Should be green
      </div>
      
      {/* Test 4: Custom Tailwind colors */}
      <div className="mb-8 p-4 bg-accent text-white rounded">
        Test 4: bg-accent (custom Tailwind) - Should be emerald green
      </div>
      
      {/* Test 5: Check CSS variables */}
      <div className="mb-8 p-4 rounded" style={{ backgroundColor: 'var(--background)', color: 'var(--foreground)', border: '2px solid var(--border)' }}>
        Test 5: Direct CSS vars - Should use theme colors
      </div>
      
      {/* Test 6: Tailwind custom color */}
      <div className="mb-8 p-4 bg-brand-emerald text-white rounded">
        Test 6: bg-brand-emerald - Should be emerald
      </div>
    </div>
  );
}

