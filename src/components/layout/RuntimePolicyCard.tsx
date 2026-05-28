import { cardClass } from '../../lib/constants';

export function RuntimePolicyCard() {
  return (
    <div className={`${cardClass} p-4`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Runtime policy</p>
      <h3 className="mt-2 text-[16px] font-semibold">Canonical provider choices</h3>
      <ul className="mt-4 space-y-3 text-[13px] leading-6 text-[var(--text-secondary)]">
        <li>
          <strong className="text-[var(--text-primary)]">Python</strong>: `pyenv` plus `pip`, `pipenv`,
          and `poetry`.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">Node.js</strong>: `Volta` as cross-platform
          runtime anchor, not `nvm`.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">Rust</strong>: `rustup` and `cargo`, first-class
          across all three desktop OS targets.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">Java</strong>: `SDKMAN!` as the preferred
          Unix-side manager, with Maven and Gradle detected as companion tooling.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">Go</strong>: official toolchain baseline first,
          with `gvm` treated as optional rather than required.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">.NET</strong>: official user-space SDK install is
          available, while `global.json` remains the project pinning surface for activation.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">PHP</strong>: `phpbrew` for managed versions, with
          Composer treated as companion tooling.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">Ruby</strong>: `rbenv` plus `ruby-build`, with
          gems and Bundler layered above the selected runtime.
        </li>
        <li>
          <strong className="text-[var(--text-primary)]">C/C++</strong>: host compiler and build toolchain
          inspection only, with `clang` / `gcc` / `MSVC` and `CMake` / `Ninja` surfaced without mutation.
        </li>
      </ul>
    </div>
  );
}
