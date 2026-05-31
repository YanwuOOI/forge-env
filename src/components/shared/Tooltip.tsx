import { useState, type ReactNode } from 'react';

interface TooltipProps {
  content: string;
  children: ReactNode;
  position?: 'top' | 'bottom';
}

export function Tooltip({ content, children, position = 'top' }: TooltipProps) {
  const [visible, setVisible] = useState(false);

  const positionClass = position === 'top'
    ? 'bottom-full mb-2 left-1/2 -translate-x-1/2'
    : 'top-full mt-2 left-1/2 -translate-x-1/2';

  return (
    <div
      className="relative inline-flex"
      onMouseEnter={() => setVisible(true)}
      onMouseLeave={() => setVisible(false)}
      onFocus={() => setVisible(true)}
      onBlur={() => setVisible(false)}
    >
      {children}
      {visible ? (
        <div
          className={`absolute z-50 whitespace-nowrap rounded-[var(--radius-sm)] bg-[var(--text-primary)] px-2 py-1 text-[11px] text-[var(--bg-canvas)] shadow-[var(--shadow-raised-sm)] ${positionClass}`}
          role="tooltip"
        >
          {content}
        </div>
      ) : null}
    </div>
  );
}
