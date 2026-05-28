import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { NavGlyph } from '../NavGlyph';
import type { NavKey } from '../../../lib/types';

const allKeys: NavKey[] = ['overview', 'hosts', 'languages', 'projects', 'deps', 'settings'];

describe('NavGlyph', () => {
  it.each(allKeys)('renders SVG for %s without crashing', (key) => {
    const { container } = render(<NavGlyph name={key} active={false} />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('uses accent color when active', () => {
    const { container } = render(<NavGlyph name="overview" active={true} />);
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('stroke')).toBe('var(--accent-primary)');
  });

  it('uses secondary color when inactive', () => {
    const { container } = render(<NavGlyph name="overview" active={false} />);
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('stroke')).toBe('var(--text-secondary)');
  });
});
