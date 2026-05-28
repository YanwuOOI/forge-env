import { describe, it, expect } from 'vitest';
import { navItems, baseDependencies } from '../constants';

describe('navItems', () => {
  it('has 6 navigation items', () => {
    expect(navItems).toHaveLength(6);
  });

  it('has unique keys', () => {
    const keys = navItems.map((item) => item.key);
    expect(new Set(keys).size).toBe(keys.length);
  });

  it('has all required fields', () => {
    for (const item of navItems) {
      expect(item.key).toBeTruthy();
      expect(item.label).toBeTruthy();
      expect(item.eyebrow).toBeTruthy();
    }
  });
});

describe('baseDependencies', () => {
  it('contains core system tools', () => {
    expect(baseDependencies).toContain('Git');
    expect(baseDependencies).toContain('curl');
    expect(baseDependencies).toContain('CMake');
  });

  it('has 10 entries', () => {
    expect(baseDependencies).toHaveLength(10);
  });
});
