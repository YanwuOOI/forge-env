interface ThemeToggleProps {
  theme: 'light' | 'dark';
  onToggle: () => void;
}

export function ThemeToggle({ theme, onToggle }: ThemeToggleProps) {
  return (
    <button
      type="button"
      onClick={onToggle}
      className="rounded-[var(--radius-md)] bg-[var(--bg-elevated)] px-3 py-2 text-[12px] font-semibold text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)] transition-[transform,box-shadow] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] active:translate-y-0 active:shadow-[var(--shadow-inset)]"
      aria-label={`Switch to ${theme === 'light' ? 'dark' : 'light'} theme`}
    >
      {theme === 'light' ? '🌙' : '☀️'}
    </button>
  );
}
