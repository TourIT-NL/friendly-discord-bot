import React from "react";

export const IconButton = ({
  icon: Icon,
  onClick,
  disabled,
  className = "",
}: {
  icon: any;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
}) => (
  <button
    onClick={onClick}
    disabled={disabled}
    className={`p-2 rounded-full hover:bg-m3-onSurface/10 active:bg-m3-onSurface/20 transition-colors disabled:opacity-30 flex items-center justify-center focus:outline-none ${className}`}
  >
    <Icon className="w-5 h-5" />
  </button>
);

export const M3Card = ({
  children,
  className = "",
  onClick,
}: {
  children: React.ReactNode;
  className?: string;
  onClick?: () => void;
}) => (
  <div
    onClick={onClick}
    className={`m3-card ${onClick ? "cursor-pointer hover:bg-m3-surfaceVariant/50 active:scale-[0.98]" : ""} ${className}`}
  >
    {children}
  </div>
);

export const SectionLabel = ({ children }: { children: React.ReactNode }) => (
  <h3 className="text-xs font-bold text-m3-primary uppercase tracking-[0.2em] mb-4 flex items-center gap-2 px-2">
    {children}
  </h3>
);
