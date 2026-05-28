import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetricTile } from '../MetricTile';

describe('MetricTile', () => {
  it('renders label and value', () => {
    render(<MetricTile label="Hosts" value="3" />);
    expect(screen.getByText('Hosts')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('renders numeric string values', () => {
    render(<MetricTile label="Runtimes" value="12" />);
    expect(screen.getByText('12')).toBeInTheDocument();
  });
});
