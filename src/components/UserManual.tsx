import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Shield, Globe, Server, Key, AlertCircle, ArrowLeft, ChevronRight } from 'lucide-react';

export const UserManual = ({ onComplete }: { onComplete: () => void }) => {
  const [step, setStep] = useState(0);
  const steps = [
    { title: "I. Core Protocols", icon: Shield, content: "This unit operates on two security layers. 'Official Gate' uses OAuth2 for public server management. 'Bypass Mode' (User Token) is required for private buffers like DMs and Friends. Use Bypass Mode for deep cleanup missions." },
    { title: "II. Developer Linkage", icon: Globe, content: "Navigate to discord.com/developers. Create an Application. Under OAuth2 > General, find your Client ID and Client Secret." },
    { title: "III. Port Authorization", icon: Server, content: "CRITICAL: You must add 'http://127.0.0.1:58123' to the Redirect URIs in your Discord Portal." },
    { title: "IV. Token Extraction", icon: Key, content: "For Bypass Mode: Open Discord in a browser. F12 > Network tab. Filter by '/api'. Find 'Authorization' header in any request." },
    { title: "V. Operational Safety", icon: AlertCircle, content: "Cleanup actions are PERMANENT. Use 'Simulation Mode' to test your range before execution. Proceed with focus." }
  ];

  return (
    <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} className="w-full max-w-2xl m3-card-elevated p-12 relative border-m3-primary/10 bg-black/40 backdrop-blur-xl">
      <div className="flex items-center justify-between mb-12">
        <div className="flex flex-col gap-2">
          <h2 className="text-4xl font-black italic uppercase tracking-tighter text-white">System Manual</h2>
          <p className="text-[10px] text-m3-primary font-black uppercase tracking-[0.4em] italic">Operational Initialization Sequence</p>
        </div>
        <div className="text-5xl font-black text-white/5 italic">0{step + 1}</div>
      </div>
      
      <div className="min-h-[220px] flex flex-col justify-center">
        <AnimatePresence mode="wait">
          <motion.div 
            key={step} 
            initial={{ opacity: 0, x: 20 }} 
            animate={{ opacity: 1, x: 0 }} 
            exit={{ opacity: 0, x: -20 }} 
            className="flex gap-8 items-start"
          >
            <div className="p-6 rounded-[2.5rem] bg-m3-primaryContainer/10 border border-m3-primary/20 text-m3-primary shadow-inner">
              {React.createElement(steps[step].icon, { className: "w-12 h-12" })}
            </div>
            <div className="flex-1 space-y-4">
              <h3 className="text-2xl font-black uppercase italic text-m3-primary tracking-tight">{steps[step].title}</h3>
              <p className="text-sm text-m3-onSurfaceVariant leading-relaxed font-bold uppercase tracking-wide opacity-90">{steps[step].content}</p>
            </div>
          </motion.div>
        </AnimatePresence>
      </div>

      <div className="mt-12 flex items-center justify-between">
        <div className="flex gap-3">
          {steps.map((_, i) => (
            <div 
              key={i} 
              className={`h-1.5 transition-all duration-500 rounded-full ${i === step ? 'w-12 bg-m3-primary' : 'w-2 bg-m3-outlineVariant'}`} 
            />
          ))}
        </div>
        <div className="flex gap-4">
          {step > 0 && (
            <button 
              onClick={() => setStep(s => s - 1)} 
              className="p-4 rounded-full border border-m3-outlineVariant text-m3-onSurfaceVariant hover:bg-white/5 transition-colors"
            >
              <ArrowLeft className="w-5 h-5" />
            </button>
          )}
          <button 
            onClick={() => step < steps.length - 1 ? setStep(s => s + 1) : onComplete()} 
            className="m3-button-primary !px-12 !py-5 shadow-xl shadow-m3-primary/20"
          >
            {step < steps.length - 1 ? "Next Phase" : "Acknowledge & Start"}
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    </motion.div>
  );
};
