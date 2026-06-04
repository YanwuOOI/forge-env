import { useState } from 'react';
import { buttonPrimaryClass, buttonSecondaryClass, cardClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';

interface WelcomeWizardProps {
  onComplete: () => void;
}

export function WelcomeWizard({ onComplete }: WelcomeWizardProps) {
  const { t } = useI18n();
  const [step, setStep] = useState(0);

  const STEPS = [
    {
      title: t('wizard.welcome'),
      description: t('wizard.welcomeDesc'),
      icon: '🔧',
    },
    {
      title: t('wizard.discover'),
      description: t('wizard.discoverDesc'),
      icon: '🔍',
    },
    {
      title: t('wizard.allSet'),
      description: t('wizard.allSetDesc'),
      icon: '🚀',
    },
  ];

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
                {t('wizard.back')}
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
              {isLast ? t('wizard.getStarted') : t('wizard.next')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
