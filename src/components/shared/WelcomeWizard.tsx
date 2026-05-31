import { useState } from 'react';
import { buttonPrimaryClass, buttonSecondaryClass, insetClass, cardClass } from '../../lib/constants';

interface WelcomeWizardProps {
  onComplete: () => void;
}

const STEPS = [
  {
    title: 'Welcome to Forge Env',
    description: 'A unified dashboard for managing language runtimes, mirrors, system dependencies, and environment configuration across all your machines.',
    icon: '🔧',
  },
  {
    title: 'Discover Your Environment',
    description: 'Forge Env automatically detects your hosts (macOS, Linux, Windows, WSL), installed runtimes, and project requirements.',
    icon: '🔍',
  },
  {
    title: 'You\'re All Set',
    description: 'Start by exploring the Overview dashboard, or jump to Languages to manage your runtime versions. You can always return to this guide from Settings.',
    icon: '🚀',
  },
];

export function WelcomeWizard({ onComplete }: WelcomeWizardProps) {
  const [step, setStep] = useState(0);
  const current = STEPS[step];
  const isLast = step === STEPS.length - 1;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--scrim)]">
      <div className={`${cardClass} mx-4 max-w-[480px] p-8`}>
        <div className="text-center">
          <span className="text-[48px]" role="img" aria-hidden="true">{current.icon}</span>
          <h2 className="mt-4 text-[22px] font-semibold text-[var(--text-primary)]">{current.title}</h2>
          <p className="mt-3 text-[14px] leading-6 text-[var(--text-secondary)]">{current.description}</p>
        </div>

        <div className="mt-8 flex items-center justify-between">
          <div className="flex gap-2">
            {STEPS.map((_, i) => (
              <div
                key={i}
                className={`size-2 rounded-full transition-colors ${
                  i === step ? 'bg-[var(--accent-primary)]' : 'bg-[var(--border-soft)]'
                }`}
              />
            ))}
          </div>

          <div className="flex gap-3">
            {step > 0 ? (
              <button
                type="button"
                className={buttonSecondaryClass}
                onClick={() => setStep(step - 1)}
              >
                Back
              </button>
            ) : null}
            <button
              type="button"
              className={buttonPrimaryClass}
              onClick={() => {
                if (isLast) {
                  onComplete();
                } else {
                  setStep(step + 1);
                }
              }}
            >
              {isLast ? 'Get started' : 'Next'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
