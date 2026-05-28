export const panelClass =
  'rounded-[var(--radius-lg)] bg-[var(--bg-panel)] shadow-[var(--shadow-raised-md)]';
export const cardClass =
  'rounded-[var(--radius-md)] bg-[var(--bg-elevated)] shadow-[var(--shadow-raised-sm)]';
export const insetClass =
  'rounded-[var(--radius-md)] bg-[var(--bg-inset)] shadow-[var(--shadow-inset)]';
export const buttonPrimaryClass =
  'rounded-[var(--radius-md)] bg-[var(--accent-soft)] px-4 py-2 text-[13px] font-semibold text-[var(--accent-primary)] shadow-[var(--shadow-raised-sm)] transition-[transform,box-shadow,background-color] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] active:translate-y-0 active:shadow-[var(--shadow-inset)]';
export const buttonSecondaryClass =
  'rounded-[var(--radius-md)] bg-[var(--bg-elevated)] px-4 py-2 text-[13px] font-semibold text-[var(--text-primary)] shadow-[var(--shadow-raised-sm)] transition-[transform,box-shadow] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] active:translate-y-0 active:shadow-[var(--shadow-inset)]';
export const buttonDisabledClass =
  'cursor-not-allowed rounded-[var(--radius-md)] bg-[rgba(127,138,154,0.18)] px-4 py-2 text-[13px] font-semibold text-[var(--text-muted)] shadow-none opacity-70';

export const navItems: { key: import('./types').NavKey; label: string; eyebrow: string }[] = [
  { key: 'overview', label: 'Overview', eyebrow: 'Command center' },
  { key: 'hosts', label: 'Hosts', eyebrow: 'Machine topology' },
  { key: 'languages', label: 'Languages', eyebrow: 'Runtime layers' },
  { key: 'projects', label: 'Projects', eyebrow: 'Project detection' },
  { key: 'deps', label: 'System Deps', eyebrow: 'Base toolchain' },
  { key: 'settings', label: 'Settings', eyebrow: 'Mirrors and export' },
];

export const baseDependencies = [
  'Git',
  'SSH',
  'OpenSSL',
  'curl',
  'wget',
  'CMake',
  'GCC',
  'Clang',
  'pkg-config',
  'FFmpeg',
];
