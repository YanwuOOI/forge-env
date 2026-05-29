import { Component, type ErrorInfo, type ReactNode } from 'react';
import { cardClass, buttonPrimaryClass, buttonSecondaryClass } from '../../lib/constants';

interface ErrorBoundaryProps {
  children: ReactNode;
  label?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error(`[ErrorBoundary:${this.props.label ?? 'unknown'}]`, error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className={`${cardClass} p-6`}>
          <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">
            {this.props.label ?? 'Component'} Error
          </p>
          <h3 className="mt-2 text-[16px] font-semibold text-[var(--danger)]">
            Something went wrong
          </h3>
          <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
            {this.state.error?.message ?? 'An unexpected error occurred while rendering this section.'}
          </p>
          <div className="mt-4 flex gap-3">
            <button type="button" className={buttonPrimaryClass} onClick={this.handleRetry}>
              Try again
            </button>
            <button
              type="button"
              className={buttonSecondaryClass}
              onClick={() => window.location.reload()}
            >
              Reload app
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
