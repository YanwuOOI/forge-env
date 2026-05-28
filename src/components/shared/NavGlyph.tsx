import type { NavKey } from '../../lib/types';

export function NavGlyph({ name, active }: { name: NavKey; active: boolean }) {
  const stroke = active ? 'var(--accent-primary)' : 'var(--text-secondary)';

  return (
    <svg
      viewBox="0 0 24 24"
      className="size-5 shrink-0"
      fill="none"
      stroke={stroke}
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {name === 'overview' ? (
        <>
          <rect x="4" y="4" width="7" height="7" rx="2" />
          <rect x="13" y="4" width="7" height="4" rx="2" />
          <rect x="13" y="10" width="7" height="10" rx="2" />
          <rect x="4" y="13" width="7" height="7" rx="2" />
        </>
      ) : null}
      {name === 'hosts' ? (
        <>
          <rect x="4" y="5" width="16" height="6" rx="2" />
          <path d="M8 11v6" />
          <path d="M16 11v6" />
          <path d="M6 17h4" />
          <path d="M14 17h4" />
        </>
      ) : null}
      {name === 'languages' ? (
        <>
          <path d="M5 7h14" />
          <path d="M8 4v16" />
          <path d="M16 4v16" />
          <path d="M5 17h14" />
        </>
      ) : null}
      {name === 'projects' ? (
        <>
          <path d="M4 7h7l2 2h7v8a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3z" />
          <path d="M9 13h6" />
        </>
      ) : null}
      {name === 'deps' ? (
        <>
          <circle cx="8" cy="12" r="3" />
          <circle cx="16" cy="12" r="3" />
          <path d="M11 12h2" />
          <path d="M8 9V6" />
          <path d="M16 15v3" />
        </>
      ) : null}
      {name === 'settings' ? (
        <>
          <circle cx="12" cy="12" r="3" />
          <path d="M12 4v2" />
          <path d="M12 18v2" />
          <path d="M4 12h2" />
          <path d="M18 12h2" />
          <path d="m6.3 6.3 1.4 1.4" />
          <path d="m16.3 16.3 1.4 1.4" />
          <path d="m17.7 6.3-1.4 1.4" />
          <path d="m7.7 16.3-1.4 1.4" />
        </>
      ) : null}
    </svg>
  );
}
