import { memo } from 'react';
import type { Dispatch, SetStateAction } from 'react';
import type {
  EnvPlan,
  ExportBundle,
  HostDetail,
  ImportAction,
  ImportResult,
  ProxySettings,
} from '../../lib/types';
import { ProxyPanel } from './ProxyPanel';
import { ExportPanel } from './ExportPanel';
import { EnvPlanPanel } from './EnvPlanPanel';
import { ImportPanel } from './ImportPanel';
import { LanguageSwitcher } from './LanguageSwitcher';

interface SettingsSectionProps {
  hostDetail: HostDetail | null;
  envPlan: EnvPlan | null;
  envTargetProfile: string;
  exportBundle: ExportBundle | null;
  importDraft: string;
  importResult: ImportResult | null;
  proxySettings: ProxySettings;
  setProxySettings: Dispatch<SetStateAction<ProxySettings>>;
  proxyPasswordDraft: string;
  setProxyPasswordDraft: Dispatch<SetStateAction<string>>;
  setEnvTargetProfile: (value: string) => void;
  setImportDraft: (value: string) => void;
  selectedImportActionIds: string[];
  setSelectedImportActionIds: (value: string[]) => void;
  onRefreshEnvPlan: () => void;
  onApplyEnvPlan: () => void;
  onExport: () => void;
  onImport: () => void;
  onApplyImport: () => void;
  onSaveProxySettings: () => void;
  onClearProxySettings: () => void;
  onRepairImportAction: (action: ImportAction) => void;
}

export const SettingsSection = memo(function SettingsSection({
  hostDetail,
  envPlan,
  envTargetProfile,
  exportBundle,
  importDraft,
  importResult,
  proxySettings,
  setProxySettings,
  proxyPasswordDraft,
  setProxyPasswordDraft,
  setEnvTargetProfile,
  setImportDraft,
  selectedImportActionIds,
  setSelectedImportActionIds,
  onRefreshEnvPlan,
  onApplyEnvPlan,
  onExport,
  onImport,
  onApplyImport,
  onSaveProxySettings,
  onClearProxySettings,
  onRepairImportAction,
}: SettingsSectionProps) {
  return (
    <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
      <div className="grid gap-4">
        <LanguageSwitcher />

        <ProxyPanel
          proxySettings={proxySettings}
          setProxySettings={setProxySettings}
          proxyPasswordDraft={proxyPasswordDraft}
          setProxyPasswordDraft={setProxyPasswordDraft}
          onSaveProxySettings={onSaveProxySettings}
          onClearProxySettings={onClearProxySettings}
        />

        <ExportPanel exportBundle={exportBundle} onExport={onExport} />

        <EnvPlanPanel
          hostDetail={hostDetail}
          envPlan={envPlan}
          envTargetProfile={envTargetProfile}
          setEnvTargetProfile={setEnvTargetProfile}
          onRefreshEnvPlan={onRefreshEnvPlan}
          onApplyEnvPlan={onApplyEnvPlan}
        />
      </div>

      <ImportPanel
        importDraft={importDraft}
        setImportDraft={setImportDraft}
        importResult={importResult}
        selectedImportActionIds={selectedImportActionIds}
        setSelectedImportActionIds={setSelectedImportActionIds}
        onImport={onImport}
        onApplyImport={onApplyImport}
        onRepairImportAction={onRepairImportAction}
      />
    </div>
  );
});
