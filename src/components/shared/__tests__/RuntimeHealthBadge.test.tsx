import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { RuntimeHealthBadge } from '../RuntimeHealthBadge';
import type { RuntimeFamilyState } from '../../../lib/types';

function makeRuntime(overrides: Partial<RuntimeFamilyState> = {}): RuntimeFamilyState {
  return {
    family: 'Python',
    provider: 'pyenv',
    providerStatus: 'ready',
    detectedBinary: '/usr/bin/python3',
    health: 'good',
    packageTools: ['pip'],
    mirrors: [],
    recommendedVersions: ['3.12'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: [],
    installed: [],
    ...overrides,
  };
}

describe('RuntimeHealthBadge', () => {
  it('displays providerStatus text', () => {
    render(<RuntimeHealthBadge runtime={makeRuntime({ providerStatus: 'ready' })} />);
    expect(screen.getByText('ready')).toBeInTheDocument();
  });

  it('displays fallback-system status', () => {
    render(<RuntimeHealthBadge runtime={makeRuntime({ providerStatus: 'fallback-system' })} />);
    expect(screen.getByText('fallback-system')).toBeInTheDocument();
  });
});
