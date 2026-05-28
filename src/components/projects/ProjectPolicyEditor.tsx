import type { ProjectRuntimePolicyOptions } from '../../lib/types';
import { insetClass } from '../../lib/constants';

interface ProjectPolicyEditorProps {
  family: string;
  options: ProjectRuntimePolicyOptions;
  onChange: (options: ProjectRuntimePolicyOptions) => void;
}

export function ProjectPolicyEditor({ family, options, onChange }: ProjectPolicyEditorProps) {
  if (family === '.NET') {
    const dotnet = options.dotnet ?? {};
    return (
      <div className="grid gap-3 md:grid-cols-[minmax(0,220px)_auto] md:items-end">
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            rollForward
          </span>
          <select
            value={dotnet.rollForward ?? ''}
            onChange={(event) =>
              onChange({
                dotnet: {
                  ...dotnet,
                  rollForward: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="disable">disable</option>
            <option value="patch">patch</option>
            <option value="feature">feature</option>
            <option value="minor">minor</option>
            <option value="major">major</option>
            <option value="latestPatch">latestPatch</option>
            <option value="latestFeature">latestFeature</option>
            <option value="latestMinor">latestMinor</option>
            <option value="latestMajor">latestMajor</option>
          </select>
        </label>
        <label className="flex items-center gap-3 rounded-[var(--radius-md)] bg-[rgba(220,231,255,0.38)] px-4 py-3 text-[12px] text-[var(--text-secondary)]">
          <input
            type="checkbox"
            checked={Boolean(dotnet.allowPrerelease)}
            onChange={(event) =>
              onChange({
                dotnet: {
                  ...dotnet,
                  allowPrerelease: event.target.checked,
                },
              })
            }
            className="size-4 rounded border border-[var(--border-soft)] accent-[var(--accent-primary)]"
          />
          <span>Allow prerelease SDK resolution</span>
        </label>
      </div>
    );
  }

  if (family === 'C/C++') {
    const cpp = options.cpp ?? {};
    return (
      <div className="grid gap-3 md:grid-cols-2">
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Generator
          </span>
          <select
            value={cpp.generator ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  generator: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="">Auto detect</option>
            <option value="Ninja">Ninja</option>
            <option value="Unix Makefiles">Unix Makefiles</option>
            <option value="NMake Makefiles">NMake Makefiles</option>
          </select>
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Build type
          </span>
          <select
            value={cpp.buildType ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  buildType: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="">Unspecified</option>
            <option value="Debug">Debug</option>
            <option value="Release">Release</option>
            <option value="RelWithDebInfo">RelWithDebInfo</option>
            <option value="MinSizeRel">MinSizeRel</option>
          </select>
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Binary dir
          </span>
          <input
            value={cpp.binaryDir ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  binaryDir: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="${sourceDir}/build/forge-env"
          />
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Toolchain file
          </span>
          <input
            value={cpp.toolchainFile ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  toolchainFile: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="cmake/toolchains/dev.cmake"
          />
        </label>
      </div>
    );
  }

  return null;
}
