import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetricPanel } from '../MetricPanel';

describe('MetricPanel', () => {
  it('renders label, value, and hint', () => {
    render(<MetricPanel label="Detected hosts" value="2" hint="Native + WSL" />);
    expect(screen.getByText('Detected hosts')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('Native + WSL')).toBeInTheDocument();
  });
});
