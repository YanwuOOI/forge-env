export const neumorphismTheme = {
  fontFamily: {
    sans: ['var(--font-ui)'],
    mono: ['var(--font-mono)'],
  },
  colors: {
    canvas: 'var(--bg-canvas)',
    panel: 'var(--bg-panel)',
    elevated: 'var(--bg-elevated)',
    inset: 'var(--bg-inset)',
    primaryText: 'var(--text-primary)',
    secondaryText: 'var(--text-secondary)',
    mutedText: 'var(--text-muted)',
    softBorder: 'var(--border-soft)',
    accent: 'var(--accent-primary)',
    accentHover: 'var(--accent-hover)',
    accentSoft: 'var(--accent-soft)',
    success: 'var(--success)',
    warning: 'var(--warning)',
    danger: 'var(--danger)',
    info: 'var(--info)',
  },
  boxShadow: {
    'neu-raised-sm': 'var(--shadow-raised-sm)',
    'neu-raised-md': 'var(--shadow-raised-md)',
    'neu-inset': 'var(--shadow-inset)',
  },
  borderRadius: {
    sm: 'var(--radius-sm)',
    md: 'var(--radius-md)',
    lg: 'var(--radius-lg)',
    pill: 'var(--radius-pill)',
  },
  fontSize: {
    12: 'var(--text-12)',
    14: 'var(--text-14)',
    16: 'var(--text-16)',
    20: 'var(--text-20)',
    24: 'var(--text-24)',
    32: 'var(--text-32)',
  },
  spacing: {
    1: 'var(--space-1)',
    2: 'var(--space-2)',
    3: 'var(--space-3)',
    4: 'var(--space-4)',
    6: 'var(--space-6)',
    8: 'var(--space-8)',
    10: 'var(--space-10)',
    12: 'var(--space-12)',
  },
  transitionDuration: {
    hover: 'var(--motion-hover)',
    press: 'var(--motion-press)',
    panel: 'var(--motion-panel-enter)',
  },
};

export const semanticComponentClasses = {
  appShell: 'bg-canvas text-primaryText min-h-dvh',
  sidebarPanel: 'rounded-lg bg-panel shadow-neu-raised-md',
  workspacePanel: 'rounded-lg bg-panel shadow-neu-raised-md',
  topBar: 'rounded-md bg-elevated shadow-neu-raised-sm',
  insetField: 'rounded-md bg-inset shadow-neu-inset',
  card: 'rounded-md bg-elevated shadow-neu-raised-sm',
  primaryButton:
    'rounded-md bg-accentSoft text-accent shadow-neu-raised-sm transition-[transform,box-shadow,background-color,color] duration-hover ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-neu-raised-md active:translate-y-0 active:shadow-neu-inset',
  secondaryButton:
    'rounded-md bg-elevated text-primaryText shadow-neu-raised-sm transition-[transform,box-shadow] duration-hover ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-neu-raised-md active:translate-y-0 active:shadow-neu-inset',
  focusRing:
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--bg-canvas)]',
};
