'use client';

import { useEffect, useState } from 'react';
import { Sun, Moon } from 'lucide-react';

export function ThemeToggle() {
  const [isDark, setIsDark] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // Check localStorage and apply theme on mount
    const savedTheme = localStorage.getItem('theme');
    const prefersDark = savedTheme === 'dark';
    
    setIsDark(prefersDark);
    setMounted(true);
    
    // Apply theme to HTML element
    if (prefersDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, []);

  const toggleTheme = () => {
    const newIsDark = !isDark;
    setIsDark(newIsDark);
    
    // Update HTML class
    if (newIsDark) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
      console.log('[Theme] Switched to DARK mode');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
      console.log('[Theme] Switched to LIGHT mode');
    }
  };

  if (!mounted) {
    return (
      <div className="w-9 h-9 rounded border border-gray-200 dark:border-gray-700" />
    );
  }

  return (
    <button
      onClick={toggleTheme}
      className="w-9 h-9 flex items-center justify-center rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800 transition-all duration-200 group"
      aria-label="Toggle theme"
      title={`Switch to ${isDark ? 'light' : 'dark'} mode`}
    >
      {isDark ? (
        <Sun className="w-4 h-4 text-gray-600 dark:text-gray-400 group-hover:rotate-45 transition-transform duration-300" />
      ) : (
        <Moon className="w-4 h-4 text-gray-600 group-hover:-rotate-12 transition-transform duration-300" />
      )}
    </button>
  );
}
