import type { ImportAction, ImportResult } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import { groupImportActionsByHost } from '../../lib/utils';

function importActionTone(status: ImportAction['status']) {
  switch (status) {
    case 'applied': return 'bg-[rgba(31,157,104,0.10)] text-[var(--success)]';
    case 'failed': return 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]';
    case 'blocked': return 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';
    case 'skipped': return 'bg-[rgba(127,138,154,0.16)] text-[var(--text-secondary)]';
    default: return 'bg-[rgba(47,107,255,0.10)] text-[var(--accent-primary)]';
  }
}

interface ImportPanelProps {
  importDraft: string;
  setImportDraft: (value: string) => void;
  importResult: ImportResult | null;
  selectedImportActionIds: string[];
  setSelectedImportActionIds: (value: string[]) => void;
  onImport: () => void;
  onApplyImport: () => void;
  onRepairImportAction: (action: ImportAction) => void;
}

export function ImportPanel({
  importDraft, setImportDraft, importResult, selectedImportActionIds,
  setSelectedImportActionIds, onImport, onApplyImport, onRepairImportAction,
}: ImportPanelProps) {
  const plannedActions = importResult?.actions.filter((a) => a.status === 'planned') ?? [];
  const plannedIds = plannedActions.map((a) => a.id);
  const selectedCount = plannedActions.filter((a) => selectedImportActionIds.includes(a.id)).length;
  const grouped = groupImportActionsByHost(importResult?.actions ?? []);

  return (
    <div className={`${cardClass} p-5`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Import reconstruction</p>
      <h3 className="mt-2 text-[18px] font-semibold">Plan host-aware rebuild actions from a bundle</h3>
      <textarea
        value={importDraft}
        onChange={(event) => setImportDraft(event.target.value)}
        placeholder="Paste an exported template bundle here"
        className={`${insetClass} mt-5 h-64 w-full resize-none border border-transparent p-4 font-mono text-[11px] leading-5 text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)] focus:ring-2 focus:ring-[var(--focus-ring)]`}
      />
      <div className="mt-4 flex flex-wrap gap-3">
        <button type="button" className={buttonSecondaryClass} onClick={onImport}>Validate import</button>
        <button type="button" className={importResult?.readyToApply && selectedCount > 0 ? buttonPrimaryClass : buttonDisabledClass} onClick={importResult?.readyToApply && selectedCount > 0 ? onApplyImport : undefined} disabled={!importResult?.readyToApply || selectedCount === 0}>Apply selected actions</button>
        <button type="button" className={plannedIds.length ? buttonSecondaryClass : buttonDisabledClass} onClick={plannedIds.length ? () => setSelectedImportActionIds(plannedIds) : undefined} disabled={!plannedIds.length}>Select all planned</button>
        <button type="button" className={selectedCount ? buttonSecondaryClass : buttonDisabledClass} onClick={selectedCount ? () => setSelectedImportActionIds([]) : undefined} disabled={!selectedCount}>Clear selection</button>
      </div>
      {importResult ? (
        <div className={`${insetClass} mt-5 p-4`}>
          <p className="text-[14px] font-semibold text-[var(--text-primary)]">{importResult.accepted ? 'Bundle accepted' : 'Bundle rejected'}</p>
          <p className="mt-2 text-[12px] text-[var(--text-secondary)]">Hosts: {importResult.hostCount} · Runtime groups: {importResult.runtimeCount} · Planned: {importResult.plannedCount} · Applied: {importResult.appliedCount}</p>
          {plannedActions.length ? <p className="mt-2 text-[12px] text-[var(--text-secondary)]">Selected now: {selectedCount} of {plannedActions.length} planned actions</p> : null}
          {!importResult.issues.length && !importResult.readyToApply && !importResult.appliedCount ? <p className="mt-3 text-[12px] text-[var(--text-secondary)]">The bundle is valid, but there are no matching host actions to apply on this machine.</p> : null}
          {importResult.issues.length ? (
            <div className="mt-4 space-y-2">
              {importResult.issues.map((issue) => <div key={issue} className="rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.10)] px-3 py-2 text-[12px] text-[var(--danger)]">{issue}</div>)}
            </div>
          ) : null}
          {importResult.actions.length ? (
            <div className="mt-4 space-y-3">
              {grouped.map(([hostLabel, actions]) => {
                const hostPlanned = actions.filter((a) => a.status === 'planned').map((a) => a.id);
                const hostSel = hostPlanned.filter((id) => selectedImportActionIds.includes(id)).length;
                return (
                  <div key={hostLabel} className="space-y-3 rounded-[var(--radius-md)] bg-[rgba(255,255,255,0.28)] p-3">
                    <div className="flex flex-wrap items-center justify-between gap-3">
                      <div>
                        <p className="text-[13px] font-semibold text-[var(--text-primary)]">{hostLabel}</p>
                        <p className="mt-1 text-[12px] text-[var(--text-secondary)]">{hostSel} selected / {hostPlanned.length} planned</p>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        <button type="button" className={hostPlanned.length ? buttonSecondaryClass : buttonDisabledClass} onClick={hostPlanned.length ? () => setSelectedImportActionIds(Array.from(new Set([...selectedImportActionIds, ...hostPlanned]))) : undefined} disabled={!hostPlanned.length}>Select host</button>
                        <button type="button" className={hostSel ? buttonSecondaryClass : buttonDisabledClass} onClick={hostSel ? () => setSelectedImportActionIds(selectedImportActionIds.filter((id) => !hostPlanned.includes(id))) : undefined} disabled={!hostSel}>Clear host</button>
                      </div>
                    </div>
                    {actions.map((action) => {
                      const canSelect = action.status === 'planned';
                      return (
                        <div key={action.id} className="rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-3 shadow-[var(--shadow-raised-sm)]">
                          <div className="flex items-start justify-between gap-3">
                            <label className={`flex items-start gap-3 ${canSelect ? 'cursor-pointer' : 'cursor-default'}`}>
                              <input type="checkbox" checked={selectedImportActionIds.includes(action.id)} disabled={!canSelect} onChange={(event) => { if (!canSelect) return; setSelectedImportActionIds(event.target.checked ? [...selectedImportActionIds, action.id] : selectedImportActionIds.filter((id) => id !== action.id)); }} className="mt-1 size-4 rounded border border-[var(--border-soft)] accent-[var(--accent-primary)]" />
                              <span>
                                <p className="text-[13px] font-semibold text-[var(--text-primary)]">{action.label}</p>
                                <p className="mt-1 text-[12px] text-[var(--text-secondary)]">{action.kind}</p>
                                {action.outcomeTitle ? <p className="mt-1 text-[12px] font-semibold text-[var(--text-primary)]">{action.outcomeTitle}</p> : null}
                              </span>
                            </label>
                            <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] ${importActionTone(action.status)}`}>{action.status}</span>
                          </div>
                          {action.reason ? <p className="mt-3 text-[12px] leading-5 text-[var(--text-secondary)]">{action.reason}</p> : null}
                          {action.remediation ? <p className="mt-2 text-[12px] leading-5 text-[var(--warning)]">Fix suggestion: {action.remediation}</p> : null}
                          {action.nextStep ? <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">Next step: {action.nextStep}</p> : null}
                          {action.status === 'blocked' && action.repairAvailable && action.family ? (
                            <div className="mt-3">
                              <button type="button" className={buttonSecondaryClass} onClick={() => onRepairImportAction(action)}>Repair provider</button>
                            </div>
                          ) : null}
                        </div>
                      );
                    })}
                  </div>
                );
              })}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
