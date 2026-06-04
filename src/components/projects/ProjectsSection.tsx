import { memo, useState } from 'react';
import type { ProjectProfile, ProjectRuntimePolicyOptions, RuntimeFamilyState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonDisabledClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';
import { ProjectPolicyEditor } from './ProjectPolicyEditor';
import { normalizeOptionalText } from '../../lib/utils';

type ProjectPolicyDraftMap = Record<string, ProjectRuntimePolicyOptions>;

interface ProjectsSectionProps {
  runtimes: RuntimeFamilyState[];
  query: string;
  setQuery: (value: string) => void;
  projects: ProjectProfile[];
  onAlign: (
    projectPath: string,
    family: string,
    version: string,
    options?: ProjectRuntimePolicyOptions,
  ) => void;
}

export const ProjectsSection = memo(function ProjectsSection({ runtimes, query, setQuery, projects, onAlign }: ProjectsSectionProps) {
  const { t } = useI18n();
  const [policyDrafts, setPolicyDrafts] = useState<ProjectPolicyDraftMap>({});
  const runtimeByFamily = new Map(runtimes.map((runtime) => [runtime.family, runtime]));

  return (
    <div className="grid gap-4">
      <div className={`${cardClass} p-5`}>
        <div className="grid gap-4 xl:grid-cols-[1fr_auto] xl:items-center">
          <div>
            <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('projects.projectDiscovery')}</p>
            <h3 className="mt-2 text-[18px] font-semibold">{t('projects.markerDriven')}</h3>
          </div>
          <div className={`${insetClass} flex items-center gap-3 px-4 py-3`}>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t('projects.searchPlaceholder')}
              className="w-full min-w-[280px] bg-transparent text-[14px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]"
            />
          </div>
        </div>
      </div>

      {projects.length ? (
        projects.map((project) => (
          <div key={project.id} className={`${cardClass} p-5`}>
            <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
              <div>
                <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{project.health}</p>
                <h3 className="mt-2 text-[18px] font-semibold">{project.name}</h3>
                <p className="mt-2 font-mono text-[12px] text-[var(--text-muted)]">{project.path}</p>
              </div>
              <div className="flex flex-wrap gap-2">
                {project.markers.map((marker) => (
                  <span
                    key={marker}
                    className="rounded-[var(--radius-pill)] bg-[var(--bg-inset)] px-3 py-1 font-mono text-[11px] text-[var(--text-secondary)] shadow-[var(--shadow-inset)]"
                  >
                    {marker}
                  </span>
                ))}
              </div>
            </div>

            <div className="mt-5 grid gap-3 xl:grid-cols-[1.1fr_0.9fr]">
              <div className={`${insetClass} p-4`}>
                <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('projects.suggestedRuntimes')}</p>
                <div className="mt-4 space-y-3">
                  {project.suggestedRuntimes.map((suggestion) => {
                    const runtime = runtimeByFamily.get(suggestion.family);
                    const manageable =
                      suggestion.family === '.NET' || suggestion.family === 'C/C++'
                        ? Boolean(runtime)
                        : Boolean(runtime?.capabilities.canInstall);
                    const policyKey = projectPolicyKey(project.id, suggestion.family);
                    const policyDraft =
                      policyDrafts[policyKey] ?? defaultProjectPolicyDraft(suggestion.family, suggestion.version);

                    return (
                      <div
                        key={`${project.id}-${suggestion.family}-${suggestion.version}`}
                        className="flex flex-col gap-3 rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-4 shadow-[var(--shadow-raised-sm)] md:flex-row md:items-center md:justify-between"
                      >
                        <div>
                          <p className="text-[14px] font-semibold text-[var(--text-primary)]">
                            {suggestion.family} · {suggestion.version}
                          </p>
                          <p className="mt-1 text-[12px] text-[var(--text-secondary)]">{suggestion.reason}</p>
                          {!manageable ? (
                            <p className="mt-2 text-[12px] leading-5 text-[var(--warning)]">
                              {t('projects.notManageable')}
                            </p>
                          ) : null}
                          {manageable && (suggestion.family === '.NET' || suggestion.family === 'C/C++') ? (
                            <div className="mt-3">
                              <ProjectPolicyEditor
                                family={suggestion.family}
                                options={policyDraft}
                                onChange={(nextOptions) =>
                                  setPolicyDrafts((current) => ({
                                    ...current,
                                    [policyKey]: nextOptions,
                                  }))
                                }
                              />
                            </div>
                          ) : null}
                        </div>
                        <button
                          type="button"
                          disabled={!manageable}
                          className={manageable ? buttonPrimaryClass : buttonDisabledClass}
                          onClick={() =>
                            onAlign(
                              project.path,
                              suggestion.family,
                              suggestion.version,
                              sanitizeProjectPolicyDraft(suggestion.family, policyDraft),
                            )
                          }
                        >
                          {projectActionLabel(suggestion.family, t)}
                        </button>
                      </div>
                    );
                  })}
                </div>
              </div>

              <div className={`${insetClass} p-4`}>
                <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('projects.riskFlags')}</p>
                {project.riskFlags.length ? (
                  <div className="mt-4 space-y-2">
                    {project.riskFlags.map((flag) => (
                      <div
                        key={flag}
                        className="rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.10)] px-3 py-3 text-[12px] text-[var(--danger)]"
                      >
                        {flag}
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="mt-4 text-[13px] leading-6 text-[var(--text-secondary)]">
                    {t('projects.noRisk')}
                  </p>
                )}
              </div>
            </div>
          </div>
        ))
      ) : (
        <div className={`${cardClass} p-6 text-center`}>
          <p className="text-[48px]" role="img" aria-hidden="true">📂</p>
          <h3 className="mt-3 text-[18px] font-semibold text-[var(--text-primary)]">{t('projects.noProjects')}</h3>
          <p className="mt-2 max-w-lg mx-auto text-[13px] leading-6 text-[var(--text-secondary)]">
            {t('projects.noProjectsHint')}
          </p>
        </div>
      )}
    </div>
  );
});

function projectActionLabel(family: string, t: (key: string) => string) {
  if (family === '.NET') return t('projects.writeSdkPin');
  if (family === 'C/C++') return t('projects.writeCmakePreset');
  return t('projects.alignRuntime');
}

function projectPolicyKey(projectId: string, family: string) {
  return `${projectId}:${family}`;
}

function defaultProjectPolicyDraft(family: string, version: string): ProjectRuntimePolicyOptions {
  if (family === '.NET') {
    return {
      dotnet: {
        rollForward: defaultDotnetRollForward(version),
        allowPrerelease: version.includes('-'),
      },
    };
  }
  if (family === 'C/C++') {
    return {
      cpp: {
        generator: '',
        binaryDir: '${sourceDir}/build/forge-env',
        toolchainFile: '',
        buildType: '',
      },
    };
  }
  return {};
}

function defaultDotnetRollForward(version: string) {
  const normalized = version.trim().replace(/ LTS$/u, '');
  return normalized.split('.').length >= 3 ? 'disable' : 'latestFeature';
}

function sanitizeProjectPolicyDraft(
  family: string,
  options?: ProjectRuntimePolicyOptions,
): ProjectRuntimePolicyOptions | undefined {
  if (!options) return undefined;

  if (family === '.NET') {
    const dotnet = options.dotnet;
    if (!dotnet) return undefined;
    return {
      dotnet: {
        rollForward: normalizeOptionalText(dotnet.rollForward),
        allowPrerelease: typeof dotnet.allowPrerelease === 'boolean' ? dotnet.allowPrerelease : undefined,
      },
    };
  }

  if (family === 'C/C++') {
    const cpp = options.cpp;
    if (!cpp) return undefined;
    return {
      cpp: {
        generator: normalizeOptionalText(cpp.generator),
        binaryDir: normalizeOptionalText(cpp.binaryDir),
        toolchainFile: normalizeOptionalText(cpp.toolchainFile),
        buildType: normalizeOptionalText(cpp.buildType),
      },
    };
  }

  return undefined;
}
