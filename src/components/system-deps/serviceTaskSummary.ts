import type { JobRecord, ServiceState } from '../../lib/types';

export interface ServiceTaskSummary {
  title: string;
  detail: string;
  nextStep: string | null;
  toneClass: string;
}

export function summarizeServiceTask(
  service: ServiceState,
  recentJob: JobRecord | undefined,
  canApplyConfig: boolean,
  reconcileLabel: string,
): ServiceTaskSummary {
  const successTone = 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]';
  const warningTone = 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';
  const dangerTone = 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]';

  if (!service.installed) {
    return {
      title: 'Not installed',
      detail: `${service.name} is not installed on the current host, so control and config reconciliation remain unavailable.`,
      nextStep: `Install ${service.name} through ${service.manager} before applying overrides.`,
      toneClass: dangerTone,
    };
  }

  if (recentJob) {
    if (recentJob.category === 'service' && recentJob.targetName === service.name) {
      const metadataTone =
        recentJob.status === 'failed'
          ? dangerTone
          : recentJob.outcomeTitle?.includes('Applied') ||
        recentJob.outcomeTitle?.includes('Started') ||
        recentJob.outcomeTitle?.includes('Restarted') ||
        recentJob.outcomeTitle?.includes('Healthy')
          ? successTone
          : recentJob.outcomeTitle?.includes('Not installed')
            ? dangerTone
            : warningTone;
      return {
        title: recentJob.outcomeTitle ?? 'Recent task',
        detail: recentJob.outcomeDetail ?? recentJob.label,
        nextStep: recentJob.nextStep ?? null,
        toneClass: metadataTone,
      };
    }

    const label = recentJob.label.toLowerCase();
    if (
      label.includes('config') &&
      (label.includes('restart') || label.includes('restarted') || label.includes('start service'))
    ) {
      return {
        title: 'Applied + reconciled',
        detail: 'The latest change wrote the managed config override and then reapplied the service process.',
        nextStep: `Verify the service is listening on ${service.port ?? 'the expected port'} and that client connections still succeed.`,
        toneClass: successTone,
      };
    }

    if (label.includes('config')) {
      return {
        title: 'Config staged',
        detail: 'The latest change updated the managed override block without reconciling the service process.',
        nextStep: canApplyConfig ? `Use ${reconcileLabel} when you want the new settings to take effect immediately.` : 'Reconcile the service manually if the new config should be applied now.',
        toneClass: warningTone,
      };
    }

    if (label.includes('stop')) {
      return {
        title: 'Stopped',
        detail: 'The latest service task stopped the process cleanly on this host.',
        nextStep: canApplyConfig ? `Use ${reconcileLabel.replace('restart', 'start')} to bring it back with the managed settings.` : 'Use Start when you want the service online again.',
        toneClass: warningTone,
      };
    }

    if (label.includes('restart') || label.includes('restarted')) {
      return {
        title: 'Restarted',
        detail: 'The latest service task recycled the running process on this host.',
        nextStep: `Verify connectivity on port ${service.port ?? 'the expected port'} and watch for any migration or bind errors in the service logs.`,
        toneClass: successTone,
      };
    }

    if (label.includes('start') || label.includes('started')) {
      return {
        title: 'Started',
        detail: 'The latest service task brought the service online on this host.',
        nextStep: `Check that the service is reachable on port ${service.port ?? 'the expected port'} from your local toolchain.`,
        toneClass: successTone,
      };
    }
  }

  if (service.running) {
    return {
      title: 'Healthy',
      detail: `${service.name} is running and detected through its expected local control path.`,
      nextStep: canApplyConfig ? `Use ${reconcileLabel} after changing port or path settings.` : 'Use Restart if you need to reload the current host state.',
      toneClass: successTone,
    };
  }

  return {
    title: 'Idle',
    detail: `${service.name} is installed but not currently running on this host.`,
    nextStep: canApplyConfig ? `Use ${reconcileLabel} to apply config changes and bring it online in one step.` : 'Use Start when you want the service online again.',
    toneClass: warningTone,
  };
}
