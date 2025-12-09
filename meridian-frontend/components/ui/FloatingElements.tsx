'use client';

import { motion } from 'framer-motion';

export function FloatingElements() {
    return (
        <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">

            {/* CSS Glass Sphere - Top Left */}
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1, y: [0, -20, 0] }}
                transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
                className="absolute top-[15%] left-[5%] w-[300px] h-[300px] rounded-full"
                style={{
                    background: 'radial-gradient(circle at 30% 30%, rgba(16, 185, 129, 0.1) 0%, rgba(5, 6, 8, 0) 70%)',
                    boxShadow: 'inset -10px -10px 20px rgba(16, 185, 129, 0.05), inset 10px 10px 20px rgba(16, 185, 129, 0.05), 0 0 50px rgba(16, 185, 129, 0.1)',
                    backdropFilter: 'blur(3px)',
                    border: '1px solid rgba(16, 185, 129, 0.1)'
                }}
            >
                {/* Inner core */}
                <div className="absolute inset-[20%] rounded-full bg-emerald-500/5 blur-xl" />
                {/* Orbital ring */}
                <div className="absolute inset-[-10%] rounded-full border border-emerald-500/10 rotate-[30deg] scale-y-50" />
            </motion.div>

            {/* SVG Molecule - Bottom Right */}
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1, rotate: [0, 5, 0] }}
                transition={{ duration: 12, repeat: Infinity, ease: "easeInOut", delay: 1 }}
                className="absolute bottom-[10%] right-[5%] w-[400px] h-[400px] opacity-60"
            >
                <svg viewBox="0 0 200 200" className="w-full h-full text-blue-500/20">
                    <defs>
                        <radialGradient id="glow" cx="50%" cy="50%" r="50%">
                            <stop offset="0%" stopColor="currentColor" stopOpacity="0.5" />
                            <stop offset="100%" stopColor="currentColor" stopOpacity="0" />
                        </radialGradient>
                    </defs>
                    <line x1="50" y1="50" x2="150" y2="150" stroke="currentColor" strokeWidth="1" />
                    <line x1="150" y1="50" x2="50" y2="150" stroke="currentColor" strokeWidth="1" />
                    <line x1="100" y1="20" x2="100" y2="180" stroke="currentColor" strokeWidth="1" />

                    <circle cx="50" cy="50" r="10" fill="url(#glow)" />
                    <circle cx="150" cy="150" r="15" fill="url(#glow)" />
                    <circle cx="150" cy="50" r="8" fill="url(#glow)" />
                    <circle cx="50" cy="150" r="12" fill="url(#glow)" />
                    <circle cx="100" cy="100" r="20" fill="url(#glow)" />
                </svg>
            </motion.div>

            {/* Glass Panel - Right */}
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1, y: [0, 15, 0] }}
                transition={{ duration: 10, repeat: Infinity, ease: "easeInOut", delay: 2 }}
                className="absolute top-[25%] right-[15%] w-[280px] h-[180px] hidden lg:block"
                style={{
                    background: 'linear-gradient(135deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0.01) 100%)',
                    backdropFilter: 'blur(10px)',
                    borderRadius: '16px',
                    border: '1px solid rgba(255,255,255,0.05)',
                    boxShadow: '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
                    transform: 'perspective(1000px) rotateY(-15deg) rotateX(5deg)'
                }}
            >
                <div className="p-6 space-y-4">
                    <div className="w-1/2 h-2 rounded-full bg-white/10" />
                    <div className="w-3/4 h-2 rounded-full bg-white/5" />
                    <div className="w-full h-20 rounded-lg bg-gradient-to-br from-emerald-500/10 to-transparent border border-emerald-500/10" />
                </div>
            </motion.div>

            {/* Glass Panel - Left */}
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1, y: [0, -15, 0] }}
                transition={{ duration: 9, repeat: Infinity, ease: "easeInOut", delay: 0.5 }}
                className="absolute bottom-[20%] left-[10%] w-[220px] h-[140px] hidden lg:block"
                style={{
                    background: 'linear-gradient(135deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0.01) 100%)',
                    backdropFilter: 'blur(10px)',
                    borderRadius: '16px',
                    border: '1px solid rgba(255,255,255,0.05)',
                    boxShadow: '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
                    transform: 'perspective(1000px) rotateY(15deg) rotateX(5deg)'
                }}
            >
                <div className="p-5 space-y-3">
                    <div className="flex gap-2">
                        <div className="w-3 h-3 rounded-full bg-red-500/20" />
                        <div className="w-3 h-3 rounded-full bg-yellow-500/20" />
                        <div className="w-3 h-3 rounded-full bg-green-500/20" />
                    </div>
                    <div className="w-full h-16 rounded-lg bg-white/5 border border-white/5" />
                </div>
            </motion.div>

        </div>
    );
}
