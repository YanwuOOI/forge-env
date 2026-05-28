import { invoke, isTauri } from '@tauri-apps/api/core';
import type {
  ProjectRuntimePolicyOptions,
  ProxySettings,
  AppPreferences,
  EnvPlan,
  ExportBundle,
  HostDetail,
  HostSummary,
  ImportResult,
  JobRecord,
  ProjectProfile,
  RuntimeFamilyState,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
  SystemDependencyState,
} from './types';

const browserHosts: HostSummary[] = [
  {
    id: 'browser-host',
    label: 'Browser Preview Host',
    kind: 'macos',
    architecture: 'arm64',
    shell: 'zsh',
    status: 'preview',
    recommendedPackageManager: 'Homebrew',
    pathPreview: ['/opt/homebrew/bin', '/usr/local/bin', '$HOME/.cargo/bin'],
  },
  {
    id: 'wsl:ubuntu-preview',
    label: 'WSL Distro · Ubuntu-24.04',
    kind: 'wsl',
    architecture: 'x86_64',
    shell: 'wsl',
    status: 'ready',
    recommendedPackageManager: 'apt',
    pathPreview: [],
  },
];

let browserRuntimes: RuntimeFamilyState[] = [
  {
    family: 'Python',
    provider: 'pyenv',
    providerStatus: 'fallback-system',
    detectedBinary: '/opt/homebrew/bin/python3',
    health: 'attention',
    packageTools: ['pip', 'pipenv', 'poetry'],
    mirrors: ['PyPI', 'Tsinghua', 'Aliyun'],
    recommendedVersions: ['3.11', '3.12', '3.13'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['Use pyenv for interpreter switching and keep project tools layered above it.'],
    installed: [
      {
        version: '3.12.4',
        channel: 'stable',
        active: true,
        source: 'pyenv',
        tools: ['pip', 'poetry'],
      },
      {
        version: '3.11.9',
        channel: 'stable',
        active: false,
        source: 'pyenv',
        tools: ['pipenv'],
      },
    ],
  },
  {
    family: 'Node.js',
    provider: 'Volta',
    providerStatus: 'fallback-system',
    detectedBinary: '/opt/homebrew/bin/node',
    health: 'attention',
    packageTools: ['npm', 'yarn', 'pnpm'],
    mirrors: ['npm', 'npmmirror'],
    recommendedVersions: ['18 LTS', '20 LTS', '22 LTS'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['Volta stays cross-platform and keeps project toolchains pinned.'],
    installed: [
      {
        version: '20.17.0',
        channel: 'lts',
        active: true,
        source: 'Volta',
        tools: ['npm', 'pnpm'],
      },
    ],
  },
  {
    family: 'Rust',
    provider: 'rustup',
    providerStatus: 'ready',
    detectedBinary: '/Users/pano/.cargo/bin/rustup',
    health: 'good',
    packageTools: ['cargo'],
    mirrors: ['crates.io', 'rsproxy'],
    recommendedVersions: ['stable', '1.84', '1.85'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['Rust toolchains are already tier-one across macOS, Linux, and Windows.'],
    installed: [
      {
        version: 'stable',
        channel: 'stable',
        active: true,
        source: 'rustup',
        tools: ['cargo'],
      },
    ],
  },
  {
    family: 'Java',
    provider: 'SDKMAN!',
    providerStatus: 'ready',
    detectedBinary: '/Users/pano/.sdkman/bin/sdkman-init.sh',
    health: 'good',
    packageTools: ['Maven', 'Gradle'],
    mirrors: ['Maven Central', 'Aliyun Maven', 'Tsinghua Maven'],
    recommendedVersions: ['8', '11', '17', '21'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['Java install, switch, and remove are available when SDKMAN! manages the host toolchain.'],
    installed: [
      {
        version: '21.0.3-tem',
        channel: 'managed',
        active: true,
        source: 'SDKMAN!',
        tools: ['Maven', 'Gradle'],
      },
    ],
  },
  {
    family: 'Go',
    provider: 'gvm',
    providerStatus: 'ready',
    detectedBinary: '/Users/pano/.gvm/scripts/gvm',
    health: 'good',
    packageTools: ['go mod'],
    mirrors: ['proxy.golang.org', 'goproxy.cn', 'goproxy.io'],
    recommendedVersions: ['1.21', '1.22', '1.23'],
    capabilities: { canInstall: true, canActivate: true, canRemove: false },
    notes: ['Go install and switch are available through gvm; per-version remove is still pending.'],
    installed: [
      {
        version: '1.22.5',
        channel: 'managed',
        active: true,
        source: 'gvm',
        tools: ['go mod'],
      },
    ],
  },
  {
    family: '.NET',
    provider: 'dotnet SDK',
    providerStatus: 'fallback-system',
    detectedBinary: '/usr/local/share/dotnet/dotnet',
    health: 'attention',
    packageTools: ['NuGet', 'dotnet restore', 'dotnet tool'],
    mirrors: ['nuget.org', 'Azure China NuGet', 'Tencent NuGet'],
    recommendedVersions: ['6.0 LTS', '8.0 LTS', '9.0'],
    capabilities: { canInstall: true, canActivate: false, canRemove: false },
    notes: ['The native preview host can install additional .NET SDKs through the official user-space installer, while activation remains project-scoped through global.json.'],
    installed: [
      {
        version: '8.0.303',
        channel: 'system',
        active: true,
        source: 'dotnet',
        tools: ['NuGet', 'dotnet restore'],
      },
      {
        version: '9.0.100',
        channel: 'system',
        active: false,
        source: 'dotnet',
        tools: ['NuGet', 'dotnet tool'],
      },
    ],
  },
  {
    family: 'PHP',
    provider: 'phpbrew',
    providerStatus: 'ready',
    detectedBinary: '/Users/pano/.phpbrew/bin/phpbrew',
    health: 'attention',
    packageTools: ['composer'],
    mirrors: ['Packagist', 'Alibaba Cloud Packagist'],
    recommendedVersions: ['8.1', '8.2', '8.3'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['The native preview host can install, activate, and remove inactive phpbrew-managed PHP versions here.'],
    installed: [
      {
        version: '8.2.24',
        channel: 'managed',
        active: true,
        source: 'phpbrew',
        tools: ['composer'],
      },
    ],
  },
  {
    family: 'Ruby',
    provider: 'rbenv',
    providerStatus: 'ready',
    detectedBinary: '/Users/pano/.rbenv/bin/rbenv',
    health: 'attention',
    packageTools: ['gem', 'bundler'],
    mirrors: ['rubygems.org', 'Ruby China'],
    recommendedVersions: ['2.7', '3.1', '3.2'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['The native preview host has rbenv plus ruby-build, so Forge Env can install, activate, and remove inactive managed Ruby versions here.'],
    installed: [
      {
        version: '3.2.2',
        channel: 'managed',
        active: true,
        source: 'rbenv',
        tools: ['gem', 'bundler'],
      },
    ],
  },
  {
    family: 'C/C++',
    provider: 'system toolchain',
    providerStatus: 'ready',
    detectedBinary: '/usr/bin/clang',
    health: 'attention',
    packageTools: ['CMake', 'Make', 'Ninja'],
    mirrors: ['system package manager'],
    recommendedVersions: ['Clang', 'GCC', 'MSVC'],
    capabilities: { canInstall: true, canActivate: false, canRemove: false },
    notes: ['The native preview host can install Clang or GCC toolchain templates through the system package manager, while activation remains project-local.'],
    installed: [
      {
        version: 'Apple clang version 16.0.0',
        channel: 'system',
        active: true,
        source: 'clang',
        tools: ['CMake', 'Ninja'],
      },
      {
        version: 'gcc version 14.1.0',
        channel: 'system',
        active: false,
        source: 'gcc',
        tools: ['Make'],
      },
    ],
  },
];

const browserWslRuntimes: RuntimeFamilyState[] = [
  {
    family: 'Python',
    provider: 'pyenv',
    providerStatus: 'ready',
    detectedBinary: '/home/pano/.pyenv/bin/pyenv',
    health: 'good',
    packageTools: ['pip', 'pipenv', 'poetry'],
    mirrors: ['PyPI', 'Tsinghua', 'Aliyun'],
    recommendedVersions: ['3.11', '3.12', '3.13'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['WSL host preview uses a managed pyenv installation for Linux-side interpreter isolation.'],
    installed: [
      {
        version: '3.12.6',
        channel: 'managed',
        active: true,
        source: 'pyenv',
        tools: ['pip', 'poetry'],
      },
    ],
  },
  {
    family: 'Node.js',
    provider: 'Volta',
    providerStatus: 'ready',
    detectedBinary: '/home/pano/.volta/bin/volta',
    health: 'good',
    packageTools: ['npm', 'yarn', 'pnpm'],
    mirrors: ['npm', 'npmmirror'],
    recommendedVersions: ['18 LTS', '20 LTS', '22 LTS'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['The WSL preview host keeps Node separate from the native preview shell.'],
    installed: [
      {
        version: '22.11.0',
        channel: 'lts',
        active: true,
        source: 'Volta',
        tools: ['npm', 'pnpm'],
      },
    ],
  },
  {
    family: 'Rust',
    provider: 'rustup',
    providerStatus: 'ready',
    detectedBinary: '/home/pano/.cargo/bin/rustup',
    health: 'good',
    packageTools: ['cargo'],
    mirrors: ['crates.io', 'rsproxy'],
    recommendedVersions: ['stable', '1.84', '1.85'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['Rust is installed inside the WSL distro and should not leak into the native host snapshot.'],
    installed: [
      {
        version: 'stable',
        channel: 'stable',
        active: true,
        source: 'rustup',
        tools: ['cargo'],
      },
    ],
  },
  {
    family: 'Java',
    provider: 'SDKMAN!',
    providerStatus: 'ready',
    detectedBinary: '/home/pano/.sdkman/bin/sdkman-init.sh',
    health: 'good',
    packageTools: ['Maven', 'Gradle'],
    mirrors: ['Maven Central', 'Aliyun Maven', 'Tsinghua Maven'],
    recommendedVersions: ['8', '11', '17', '21'],
    capabilities: { canInstall: true, canActivate: true, canRemove: true },
    notes: ['Java is managed through SDKMAN! in the Linux host preview.'],
    installed: [
      {
        version: '17.0.12-tem',
        channel: 'managed',
        active: true,
        source: 'SDKMAN!',
        tools: ['Maven', 'Gradle'],
      },
    ],
  },
  {
    family: 'Go',
    provider: 'gvm',
    providerStatus: 'ready',
    detectedBinary: '/home/pano/.gvm/scripts/gvm',
    health: 'good',
    packageTools: ['go mod'],
    mirrors: ['proxy.golang.org', 'goproxy.cn', 'goproxy.io'],
    recommendedVersions: ['1.21', '1.22', '1.23'],
    capabilities: { canInstall: true, canActivate: true, canRemove: false },
    notes: ['The Linux-side Go toolchain is managed independently with gvm.'],
    installed: [
      {
        version: '1.23.1',
        channel: 'managed',
        active: true,
        source: 'gvm',
        tools: ['go mod'],
      },
    ],
  },
  {
    family: '.NET',
    provider: 'dotnet SDK',
    providerStatus: 'fallback-system',
    detectedBinary: '/usr/bin/dotnet',
    health: 'attention',
    packageTools: ['NuGet', 'dotnet restore', 'dotnet tool'],
    mirrors: ['nuget.org', 'Azure China NuGet', 'Tencent NuGet'],
    recommendedVersions: ['6.0 LTS', '8.0 LTS', '9.0'],
    capabilities: { canInstall: true, canActivate: false, canRemove: false },
    notes: ['The WSL preview host can install additional .NET SDKs through the official user-space installer, while activation remains project-scoped through global.json.'],
    installed: [
      {
        version: '8.0.404',
        channel: 'system',
        active: true,
        source: 'dotnet',
        tools: ['NuGet', 'dotnet restore'],
      },
    ],
  },
  {
    family: 'PHP',
    provider: 'phpbrew',
    providerStatus: 'fallback-system',
    detectedBinary: '/usr/bin/php',
    health: 'attention',
    packageTools: ['composer'],
    mirrors: ['Packagist', 'Alibaba Cloud Packagist'],
    recommendedVersions: ['8.1', '8.2', '8.3'],
    capabilities: { canInstall: false, canActivate: false, canRemove: false },
    notes: ['The WSL preview host only exposes a system PHP runtime right now.'],
    installed: [
      {
        version: '8.2.10',
        channel: 'system',
        active: true,
        source: 'system',
        tools: ['composer'],
      },
    ],
  },
  {
    family: 'Ruby',
    provider: 'rbenv',
    providerStatus: 'fallback-system',
    detectedBinary: '/usr/bin/ruby',
    health: 'attention',
    packageTools: ['gem', 'bundler'],
    mirrors: ['rubygems.org', 'Ruby China'],
    recommendedVersions: ['2.7', '3.1', '3.2'],
    capabilities: { canInstall: false, canActivate: false, canRemove: false },
    notes: ['The WSL preview host exposes a system Ruby runtime, but no managed rbenv installation yet.'],
    installed: [
      {
        version: '3.0.2p107',
        channel: 'system',
        active: true,
        source: 'system',
        tools: ['gem', 'bundler'],
      },
    ],
  },
  {
    family: 'C/C++',
    provider: 'system toolchain',
    providerStatus: 'ready',
    detectedBinary: '/usr/bin/clang',
    health: 'attention',
    packageTools: ['CMake', 'Make', 'Ninja'],
    mirrors: ['system package manager'],
    recommendedVersions: ['Clang', 'GCC', 'MSVC'],
    capabilities: { canInstall: true, canActivate: false, canRemove: false },
    notes: ['The Linux preview host can install Clang or GCC toolchain templates through the system package manager, while activation remains project-local.'],
    installed: [
      {
        version: 'Ubuntu clang version 18.1.3',
        channel: 'system',
        active: true,
        source: 'clang',
        tools: ['CMake', 'Ninja'],
      },
      {
        version: 'gcc version 13.2.0',
        channel: 'system',
        active: false,
        source: 'gcc',
        tools: ['Make'],
      },
    ],
  },
];

const browserRuntimeInventory: Record<string, RuntimeFamilyState[]> = {
  'browser-host': browserRuntimes,
  'wsl:ubuntu-preview': browserWslRuntimes,
};

const browserDependencyInventory: Record<string, SystemDependencyState[]> = {
  'browser-host': [
    {
      name: 'Git',
      command: 'git',
      installed: true,
      version: 'git version 2.45.1',
      sourceHint: 'Required for source control and most language installers.',
    },
    {
      name: 'OpenSSL',
      command: 'openssl',
      installed: true,
      version: 'OpenSSL 3.x',
      sourceHint: 'Common TLS dependency for language builds.',
    },
  ],
  'wsl:ubuntu-preview': [
    {
      name: 'Git',
      command: 'git',
      installed: true,
      version: 'git version 2.43.0',
      sourceHint: 'Required for repository work inside the distro.',
    },
    {
      name: 'pkg-config',
      command: 'pkg-config',
      installed: true,
      version: '1.8.1',
      sourceHint: 'Common Linux build dependency for native language extensions.',
    },
    {
      name: 'FFmpeg',
      command: 'ffmpeg',
      installed: false,
      version: null,
      sourceHint: 'Optional media dependency that is still missing in the preview distro.',
    },
  ],
};

const browserServiceInventory: Record<string, ServiceState[]> = {
  'browser-host': [
    {
      name: 'Redis',
      kind: 'cache',
      installed: true,
      running: true,
      health: 'good',
      version: 'Redis server v=7.2.5',
      manager: 'Homebrew services',
      port: 6379,
      dataDir: '/opt/homebrew/var/db/redis',
      notes: ['Local cache service is reachable through redis-cli ping.'],
    },
    {
      name: 'PostgreSQL',
      kind: 'database',
      installed: true,
      running: false,
      health: 'attention',
      version: 'postgres (PostgreSQL) 16.4',
      manager: 'Homebrew services',
      port: 5432,
      dataDir: '/opt/homebrew/var/postgresql@16',
      notes: ['Installed but not currently accepting connections.'],
    },
    {
      name: 'MySQL',
      kind: 'database',
      installed: false,
      running: false,
      health: 'missing',
      version: null,
      manager: 'Homebrew services',
      port: 3306,
      dataDir: null,
      notes: ['MySQL binaries are not present in the browser preview host.'],
    },
    {
      name: 'MongoDB',
      kind: 'database',
      installed: true,
      running: false,
      health: 'attention',
      version: 'db version v8.0.1',
      manager: 'Homebrew services',
      port: 27017,
      dataDir: '/opt/homebrew/var/mongodb',
      notes: ['MongoDB is installed but currently stopped on the preview host.'],
    },
    {
      name: 'RabbitMQ',
      kind: 'queue',
      installed: false,
      running: false,
      health: 'missing',
      version: null,
      manager: 'Homebrew services',
      port: 5672,
      dataDir: null,
      notes: ['RabbitMQ is not installed on the browser preview host yet.'],
    },
    {
      name: 'Nginx',
      kind: 'proxy',
      installed: true,
      running: true,
      health: 'good',
      version: 'nginx version: nginx/1.27.2',
      manager: 'Homebrew services',
      port: 8080,
      dataDir: null,
      notes: ['Nginx is running as the lightweight local edge proxy in preview mode.'],
    },
  ],
  'wsl:ubuntu-preview': [
    {
      name: 'Redis',
      kind: 'cache',
      installed: true,
      running: false,
      health: 'attention',
      version: 'Redis server v=7.0.15',
      manager: 'systemctl',
      port: 6379,
      dataDir: '/var/lib/redis',
      notes: ['Redis is installed in the distro but currently stopped.'],
    },
    {
      name: 'PostgreSQL',
      kind: 'database',
      installed: true,
      running: true,
      health: 'good',
      version: 'postgres (PostgreSQL) 15.8',
      manager: 'systemctl',
      port: 5432,
      dataDir: '/var/lib/postgresql/15/main',
      notes: ['PostgreSQL is active inside the preview distro.'],
    },
    {
      name: 'MySQL',
      kind: 'database',
      installed: true,
      running: false,
      health: 'attention',
      version: 'mysqld  Ver 8.0.39',
      manager: 'systemctl',
      port: 3306,
      dataDir: '/var/lib/mysql',
      notes: ['MySQL needs to be started through the service manager.'],
    },
    {
      name: 'MongoDB',
      kind: 'database',
      installed: false,
      running: false,
      health: 'missing',
      version: null,
      manager: 'systemctl',
      port: 27017,
      dataDir: '/var/lib/mongodb',
      notes: ['MongoDB has not been installed in the preview distro yet.'],
    },
    {
      name: 'RabbitMQ',
      kind: 'queue',
      installed: true,
      running: false,
      health: 'attention',
      version: '3.13.7',
      manager: 'systemctl',
      port: 5672,
      dataDir: '/var/lib/rabbitmq',
      notes: ['RabbitMQ is installed in the distro but currently stopped.'],
    },
    {
      name: 'Nginx',
      kind: 'proxy',
      installed: true,
      running: true,
      health: 'good',
      version: 'nginx version: nginx/1.24.0',
      manager: 'systemctl',
      port: 80,
      dataDir: null,
      notes: ['Nginx is active inside the preview distro.'],
    },
  ],
};

const browserServiceConfigInventory: Record<string, ServiceConfigState[]> = {
  'browser-host': [
    {
      serviceName: 'Redis',
      configPath: '/opt/homebrew/etc/redis.conf',
      port: 6379,
      dataDir: '/opt/homebrew/var/db/redis',
      canEditPort: true,
      canEditDataDir: true,
      notes: ['Forge Env can append a managed Redis override block and keep the rest of redis.conf untouched.'],
    },
    {
      serviceName: 'PostgreSQL',
      configPath: '/opt/homebrew/var/postgresql@16/postgresql.conf',
      port: 5432,
      dataDir: '/opt/homebrew/var/postgresql@16',
      canEditPort: true,
      canEditDataDir: false,
      notes: ['Port overrides are managed in postgresql.conf; data directory migration remains manual.'],
    },
    {
      serviceName: 'MySQL',
      configPath: '/opt/homebrew/etc/my.cnf',
      port: 3306,
      dataDir: null,
      canEditPort: true,
      canEditDataDir: false,
      notes: ['A managed [mysqld] block can override the port when a writable config file exists.'],
    },
    {
      serviceName: 'MongoDB',
      configPath: '/opt/homebrew/etc/mongod.conf',
      port: 27017,
      dataDir: '/opt/homebrew/var/mongodb',
      canEditPort: false,
      canEditDataDir: false,
      notes: ['MongoDB config is preview-only for now because Forge Env does not rewrite mongod.conf yet.'],
    },
    {
      serviceName: 'RabbitMQ',
      configPath: '/opt/homebrew/etc/rabbitmq/rabbitmq.conf',
      port: 5672,
      dataDir: null,
      canEditPort: false,
      canEditDataDir: false,
      notes: ['RabbitMQ config is preview-only for now.'],
    },
    {
      serviceName: 'Nginx',
      configPath: '/opt/homebrew/etc/nginx/nginx.conf',
      port: 8080,
      dataDir: null,
      canEditPort: false,
      canEditDataDir: false,
      notes: ['Nginx config preview is available, but Forge Env does not mutate nginx.conf yet.'],
    },
  ],
  'wsl:ubuntu-preview': [
    {
      serviceName: 'Redis',
      configPath: '/etc/redis/redis.conf',
      port: 6379,
      dataDir: '/var/lib/redis',
      canEditPort: true,
      canEditDataDir: true,
      notes: ['Redis can be re-pointed inside the distro, but directory permissions still need to match the service user.'],
    },
    {
      serviceName: 'PostgreSQL',
      configPath: '/etc/postgresql/15/main/postgresql.conf',
      port: 5432,
      dataDir: '/var/lib/postgresql/15/main',
      canEditPort: true,
      canEditDataDir: false,
      notes: ['The Linux preview host exposes the distro postgresql.conf path for host-aware overrides.'],
    },
    {
      serviceName: 'MySQL',
      configPath: '/etc/mysql/mysql.conf.d/mysqld.cnf',
      port: 3306,
      dataDir: '/var/lib/mysql',
      canEditPort: true,
      canEditDataDir: false,
      notes: ['MySQL data directory changes remain read-only in preview mode.'],
    },
    {
      serviceName: 'MongoDB',
      configPath: '/etc/mongod.conf',
      port: 27017,
      dataDir: '/var/lib/mongodb',
      canEditPort: false,
      canEditDataDir: false,
      notes: ['MongoDB config stays read-only in preview mode.'],
    },
    {
      serviceName: 'RabbitMQ',
      configPath: '/etc/rabbitmq/rabbitmq.conf',
      port: 5672,
      dataDir: '/var/lib/rabbitmq',
      canEditPort: false,
      canEditDataDir: false,
      notes: ['RabbitMQ config stays read-only in preview mode.'],
    },
    {
      serviceName: 'Nginx',
      configPath: '/etc/nginx/nginx.conf',
      port: 80,
      dataDir: null,
      canEditPort: false,
      canEditDataDir: false,
      notes: ['Nginx config stays read-only in preview mode.'],
    },
  ],
};

const browserServiceArtifactInventory: Record<string, ServiceArtifact[]> = {
  'browser-host': [
    {
      serviceName: 'Redis',
      kind: 'logical-backup',
      path: '/Users/pano/.forge-env/service-artifacts/redis/logical-backup/redis-logical-1716287800.sql.gz',
      sizeBytes: 4096,
      createdAt: '1716287800',
      managed: true,
    },
    {
      serviceName: 'Redis',
      kind: 'backup',
      path: '/Users/pano/.forge-env/service-artifacts/redis/backup/redis-backup-1716288000.tar.gz',
      sizeBytes: 18432,
      createdAt: '1716288000',
      managed: true,
    },
    {
      serviceName: 'PostgreSQL',
      kind: 'export',
      path: '/Users/pano/.forge-env/service-artifacts/postgresql/export/postgresql-export-1716288600.tar.gz',
      sizeBytes: 524288,
      createdAt: '1716288600',
      managed: true,
    },
  ],
  'wsl:ubuntu-preview': [
    {
      serviceName: 'MySQL',
      kind: 'logical-backup',
      path: '/home/pano/.forge-env/service-artifacts/mysql/logical-backup/mysql-logical-1716289100.sql.gz',
      sizeBytes: 98304,
      createdAt: '1716289100',
      managed: true,
    },
    {
      serviceName: 'MySQL',
      kind: 'backup',
      path: '/home/pano/.forge-env/service-artifacts/mysql/backup/mysql-backup-1716289200.tar.gz',
      sizeBytes: 262144,
      createdAt: '1716289200',
      managed: true,
    },
  ],
};

let browserJobs: JobRecord[] = [
  {
    id: 'job-seed-1',
    label: 'Initial project scan',
    status: 'completed',
    timestamp: new Date().toISOString(),
  },
];
let browserMirrorPreset: string | null = null;
let browserSelectedHostId: string | null = null;
let browserProxySettings: ProxySettings = {
  enabled: false,
  scheme: 'http',
  host: '',
  port: '',
  username: '',
  passwordSaved: false,
  secureStore: 'Keychain',
};
const browserEnvPlan: EnvPlan = {
  targetProfile: '~/.zshrc',
  availableProfiles: [
    { path: '~/.zshrc', exists: true, managedByForgeEnv: false },
    { path: '~/.zprofile', exists: true, managedByForgeEnv: false },
  ],
  pathEntries: ['$VOLTA_HOME/bin', '$CARGO_HOME/bin'],
  variables: [
    {
      key: 'VOLTA_HOME',
      value: '$HOME/.volta',
      reason: 'Volta installs its shims and cache under this directory.',
    },
    {
      key: 'CARGO_HOME',
      value: '$HOME/.cargo',
      reason: 'Cargo binaries and install metadata live under this directory.',
    },
  ],
  managedBlock:
    '# >>> forge-env env >>>\n# Generated by Forge Env. Edit via the app so changes stay in sync.\nexport VOLTA_HOME="$HOME/.volta"\nexport CARGO_HOME="$HOME/.cargo"\nexport PATH="$VOLTA_HOME/bin:$CARGO_HOME/bin:$PATH"\n# <<< forge-env env <<<',
  notes: [
    'Forge Env writes a managed shell block and leaves the rest of your profile untouched.',
    'PATH entries are prepended so shimmed toolchains win over system fallbacks.',
  ],
};

const browserWslEnvPlan: EnvPlan = {
  targetProfile: '/home/pano/.bashrc',
  availableProfiles: [
    { path: '/home/pano/.bashrc', exists: true, managedByForgeEnv: false },
    { path: '/home/pano/.profile', exists: true, managedByForgeEnv: false },
  ],
  pathEntries: ['$PYENV_ROOT/bin', '$VOLTA_HOME/bin', '$CARGO_HOME/bin'],
  variables: [
    {
      key: 'PYENV_ROOT',
      value: '$HOME/.pyenv',
      reason: 'pyenv stores Linux-side interpreters and shims here.',
    },
    {
      key: 'VOLTA_HOME',
      value: '$HOME/.volta',
      reason: 'Volta remains scoped to the WSL distro when applied on this host.',
    },
    {
      key: 'CARGO_HOME',
      value: '$HOME/.cargo',
      reason: 'Rust toolchains are installed inside the distro home directory.',
    },
  ],
  managedBlock:
    '# >>> forge-env env >>>\n# Generated by Forge Env. Edit via the app so changes stay in sync.\nexport PYENV_ROOT="$HOME/.pyenv"\nexport VOLTA_HOME="$HOME/.volta"\nexport CARGO_HOME="$HOME/.cargo"\nexport PATH="$PYENV_ROOT/bin:$VOLTA_HOME/bin:$CARGO_HOME/bin:$PATH"\n# <<< forge-env env <<<',
  notes: [
    'This preview plan targets a Linux shell profile inside WSL.',
    'Host-aware env plans keep Windows native PATH and WSL PATH isolated from each other.',
  ],
};

const browserProjects: ProjectProfile[] = [
  {
    id: 'sample-project-1',
    name: 'workspace-api',
    path: '~/code/workspace-api',
    markers: ['pyproject.toml', '.python-version'],
    suggestedRuntimes: [
      { family: 'Python', version: '3.12', reason: 'Detected pyproject-based Python service.' },
    ],
    riskFlags: ['poetry missing'],
    health: 'attention',
  },
  {
    id: 'sample-project-2',
    name: 'desktop-shell',
    path: '~/code/desktop-shell',
    markers: ['package.json', '.nvmrc', 'Cargo.toml'],
    suggestedRuntimes: [
      { family: 'Node.js', version: '20 LTS', reason: 'Frontend package.json plus .nvmrc found.' },
      { family: 'Rust', version: 'stable', reason: 'Cargo.toml detected for Tauri shell.' },
    ],
    riskFlags: [],
    health: 'good',
  },
  {
    id: 'sample-project-3',
    name: 'platform-services',
    path: '~/code/platform-services',
    markers: ['pom.xml', 'go.mod'],
    suggestedRuntimes: [
      { family: 'Java', version: '21', reason: 'pom.xml detected for JVM service build.' },
      { family: 'Go', version: '1.22', reason: 'go.mod detected for Go CLI or service code.' },
    ],
    riskFlags: [],
    health: 'good',
  },
  {
    id: 'sample-project-4',
    name: 'ops-worker',
    path: '~/code/ops-worker',
    markers: ['global.json', '*.csproj', '*.sln'],
    suggestedRuntimes: [
      { family: '.NET', version: '8.0 LTS', reason: 'Solution and SDK pin markers detected for a .NET service.' },
    ],
    riskFlags: [],
    health: 'good',
  },
  {
    id: 'sample-project-5',
    name: 'legacy-cms',
    path: '~/code/legacy-cms',
    markers: ['composer.json'],
    suggestedRuntimes: [
      { family: 'PHP', version: '8.2', reason: 'composer.json detected for a PHP application.' },
    ],
    riskFlags: ['composer-only stack may still rely on a separate frontend asset pipeline.'],
    health: 'attention',
  },
  {
    id: 'sample-project-6',
    name: 'ops-automation',
    path: '~/code/ops-automation',
    markers: ['Gemfile', '.ruby-version'],
    suggestedRuntimes: [
      { family: 'Ruby', version: '3.2', reason: 'Bundler and Ruby version markers detected.' },
    ],
    riskFlags: [],
    health: 'good',
  },
  {
    id: 'sample-project-7',
    name: 'render-core',
    path: '~/code/render-core',
    markers: ['CMakeLists.txt', 'compile_commands.json'],
    suggestedRuntimes: [
      { family: 'C/C++', version: 'system toolchain', reason: 'Native build markers detected for a C/C++ workspace.' },
    ],
    riskFlags: [],
    health: 'good',
  },
];

function seedJob(
  label: string,
  family?: string,
  version?: string,
  extra?: Partial<JobRecord>,
): JobRecord {
  return {
    id: `job-${Date.now()}`,
    label,
    status: 'completed',
    family,
    version,
    category: null,
    targetName: null,
    outcomeTitle: null,
    outcomeDetail: null,
    nextStep: null,
    timestamp: new Date().toISOString(),
    ...extra,
  };
}

function cloneRuntimeList(hostId: string): RuntimeFamilyState[] {
  return JSON.parse(
    JSON.stringify(browserRuntimeInventory[hostId] ?? browserRuntimeInventory['browser-host']),
  ) as RuntimeFamilyState[];
}

function cloneDependencyList(hostId: string): SystemDependencyState[] {
  return JSON.parse(
    JSON.stringify(browserDependencyInventory[hostId] ?? browserDependencyInventory['browser-host']),
  ) as SystemDependencyState[];
}

function cloneServiceList(hostId: string): ServiceState[] {
  return JSON.parse(
    JSON.stringify(browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']),
  ) as ServiceState[];
}

function cloneServiceConfigList(hostId: string): ServiceConfigState[] {
  return JSON.parse(
    JSON.stringify(
      browserServiceConfigInventory[hostId] ?? browserServiceConfigInventory['browser-host'],
    ),
  ) as ServiceConfigState[];
}

function cloneServiceArtifactList(hostId: string): ServiceArtifact[] {
  return JSON.parse(
    JSON.stringify(
      browserServiceArtifactInventory[hostId] ?? browserServiceArtifactInventory['browser-host'] ?? [],
    ),
  ) as ServiceArtifact[];
}

function cloneEnvPlan(hostId: string): EnvPlan {
  const source = hostId === 'wsl:ubuntu-preview' ? browserWslEnvPlan : browserEnvPlan;
  return JSON.parse(JSON.stringify(source)) as EnvPlan;
}

async function browserInvoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  switch (command) {
    case 'hosts_list':
      return browserHosts as T;
    case 'hosts_inspect':
      browserSelectedHostId = String(args.hostId ?? browserHosts[0].id);
      return (String(args.hostId ?? '') === 'wsl:ubuntu-preview'
        ? {
            summary: browserHosts[1],
            osVersion: 'Ubuntu 24.04.1 LTS',
            cwd: '/home/pano/workspace',
            homeDir: '/home/pano',
            pathEntriesCount: 8,
            shellProfiles: ['/home/pano/.bashrc', '/home/pano/.profile'],
            packageManagers: ['apt'],
            mirrorsSupported: ['npm', 'pip', 'cargo', 'system-package-manager'],
            notes: [
              'Browser preview mode includes a sample WSL host to exercise the host model.',
              'In a real Windows build, WSL distros are discovered through wsl.exe and inspected independently.',
            ],
          }
        : {
            summary: browserHosts[0],
            osVersion: 'macOS Preview',
            cwd: '~/preview',
            homeDir: '~',
            pathEntriesCount: 3,
            shellProfiles: ['~/.zshrc', '~/.zprofile'],
            packageManagers: ['Homebrew'],
            mirrorsSupported: ['npm', 'pip', 'cargo', 'homebrew'],
            notes: [
              'Browser preview mode uses sample host data.',
              'Real Tauri runs will return OS-specific PATH and package manager hints.',
            ],
          }) as T;
    case 'projects_inspect':
      return browserProjects as T;
    case 'project_runtime_apply': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const family = String(args.family ?? '');
      const version = String(args.version ?? '');
      const projectPath = String(args.projectPath ?? '');
      const options = (args.options ?? null) as ProjectRuntimePolicyOptions | null;
      const runtime = (browserRuntimeInventory[hostId] ?? browserRuntimeInventory['browser-host']).find(
        (entry) => entry.family === family,
      );
      if ((family === '.NET' || family === 'C/C++') && runtime) {
        const optionSuffix =
          family === '.NET'
            ? describeDotnetPolicyOptions(options?.dotnet)
            : describeCppPolicyOptions(options?.cpp);
        const job = seedJob(`Apply ${family} project policy`, family, version, {
          category: 'project-policy',
          targetName: family,
          outcomeTitle: family === '.NET' ? 'Project SDK pinned' : 'Project preset written',
          outcomeDetail:
            family === '.NET'
              ? `${projectPath} would receive a Forge Env managed global.json SDK pin for ${version}${optionSuffix} in preview mode.`
              : `${projectPath} would receive a Forge Env managed CMakePresets.json compiler/build preset for ${version}${optionSuffix} in preview mode.`,
          nextStep:
            family === '.NET'
              ? 'Reopen the project shell or rerun dotnet restore so the pinned SDK is picked up.'
              : 'Reconfigure the project through CMakePresets.json so the compiler hint is picked up.',
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      const job = seedJob(`Apply ${family} project policy`, family, version, {
        status: 'failed',
        category: 'project-policy',
        targetName: family,
        outcomeTitle: 'Unsupported project policy',
        outcomeDetail: `${family} does not expose a project-level policy mutation in preview mode.`,
        nextStep: 'Use project policy apply only for .NET SDK pins or C/C++ CMake presets for now.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'runtimes_list':
      return cloneRuntimeList(String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id)) as T;
    case 'deps_list':
      return cloneDependencyList(String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id)) as T;
    case 'services_list':
      return cloneServiceList(String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id)) as T;
    case 'services_config_list':
      return cloneServiceConfigList(
        String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id),
      ) as T;
    case 'service_artifacts_list':
      return cloneServiceArtifactList(
        String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id),
      ) as T;
    case 'jobs_subscribe':
      return browserJobs as T;
    case 'runtimes_install': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const family = String(args.family ?? '');
      const version = String(args.version ?? '');
      const runtime = (browserRuntimeInventory[hostId] ?? browserRuntimeInventory['browser-host']).find(
        (entry) => entry.family === family,
      );
      if (runtime && !runtime.capabilities.canInstall) {
        const job = seedJob(`Install ${family} ${version}`, family, version, {
          status: 'failed',
          category: 'runtime',
          targetName: family,
          outcomeTitle: 'Unsupported operation',
          outcomeDetail: `${family} is inspect-only in preview mode and does not expose install yet.`,
          nextStep: `Use a family with install capability or wait for Forge Env to wire ${family} installs.`,
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      if (runtime && !runtime.installed.some((entry) => entry.version === version)) {
        runtime.installed.push({
          version,
          channel: 'stable',
          active: false,
          source: runtime.provider,
          tools: runtime.packageTools.slice(0, 2),
        });
      }
      const job = seedJob(`Install ${family} ${version}`, family, version, {
        category: 'runtime',
        targetName: family,
        outcomeTitle: 'Installed',
        outcomeDetail: `${family} ${version} was installed through its canonical provider in preview mode.`,
        nextStep: runtime?.capabilities.canActivate
          ? `Activate ${family} ${version} if it should become the default toolchain.`
          : family === '.NET'
            ? 'Use global.json or the managed environment block if this SDK should become the project default.'
            : `Review the installed ${family} toolchain and apply any shell or project-level pinning needed before use.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'runtimes_switch': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const family = String(args.family ?? '');
      const version = String(args.version ?? '');
      const runtime = (browserRuntimeInventory[hostId] ?? browserRuntimeInventory['browser-host']).find(
        (entry) => entry.family === family,
      );
      if (runtime && !runtime.capabilities.canActivate) {
        const job = seedJob(`Activate ${family} ${version}`, family, version, {
          status: 'failed',
          category: 'runtime',
          targetName: family,
          outcomeTitle: 'Unsupported operation',
          outcomeDetail: `${family} is inspect-only in preview mode and does not expose activation yet.`,
          nextStep: `Wait for Forge Env to wire ${family} activation before retrying.`,
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      runtime?.installed.forEach((entry) => {
        entry.active = entry.version === version;
      });
      const job = seedJob(`Activate ${family} ${version}`, family, version, {
        category: 'runtime',
        targetName: family,
        outcomeTitle: 'Activated',
        outcomeDetail: `${family} ${version} was set as the active toolchain in preview mode.`,
        nextStep: `Refresh shells or environment blocks if ${family} should win on PATH immediately.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'runtimes_remove': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const family = String(args.family ?? '');
      const version = String(args.version ?? '');
      const runtime = (browserRuntimeInventory[hostId] ?? browserRuntimeInventory['browser-host']).find(
        (entry) => entry.family === family,
      );
      if (runtime && !runtime.capabilities.canRemove) {
        const job = seedJob(`Remove ${family} ${version}`, family, version, {
          status: 'failed',
          category: 'runtime',
          targetName: family,
          outcomeTitle: 'Unsupported operation',
          outcomeDetail: `${family} does not expose remove in preview mode.`,
          nextStep: `Use a runtime family with remove capability or wait for Forge Env to wire ${family} removal.`,
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      if (runtime) {
        runtime.installed = runtime.installed.filter((entry) => entry.version !== version);
      }
      const job = seedJob(`Remove ${family} ${version}`, family, version, {
        category: 'runtime',
        targetName: family,
        outcomeTitle: 'Removed',
        outcomeDetail: `${family} ${version} was removed from the preview provider inventory.`,
        nextStep: `Install or activate another ${family} version if projects still depend on it.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'deps_install': {
      const deps = Array.isArray(args.dependencies) ? String(args.dependencies) : 'dependencies';
      const job = seedJob(`Install base dependencies: ${deps}`, undefined, undefined, {
        category: 'dependency',
        targetName: 'base-template',
        outcomeTitle: 'Dependencies installed',
        outcomeDetail: `The base dependency template was sent to the preview host package manager.`,
        nextStep: 'Verify any privileged package operations completed successfully on the target host.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_action': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      const action = String(args.action ?? 'restart');
      const service = (browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']).find(
        (entry) => entry.name === name,
      );
      if (service) {
        if (action === 'start') {
          service.running = true;
          service.health = service.installed ? 'good' : service.health;
        } else if (action === 'stop') {
          service.running = false;
          service.health = service.installed ? 'attention' : service.health;
        }
      }
      const job = seedJob(`${action} ${name} on ${hostId}`, undefined, undefined, {
        category: 'service',
        targetName: name,
        outcomeTitle: action === 'start' ? 'Started' : action === 'stop' ? 'Stopped' : 'Restarted',
        outcomeDetail:
          action === 'start'
            ? `${name} was started on ${hostId}.`
            : action === 'stop'
              ? `${name} was stopped on ${hostId}.`
              : `${name} was restarted on ${hostId}.`,
        nextStep:
          action === 'stop'
            ? `Start ${name} again when dependent projects need it online.`
            : `Verify that ${name} is reachable on its expected local port.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_config_apply': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      const port =
        typeof args.port === 'number'
          ? args.port
          : args.port == null || args.port === ''
            ? null
            : Number(args.port);
      const dataDir =
        typeof args.dataDir === 'string' && args.dataDir.trim().length ? args.dataDir.trim() : null;
      const config = (browserServiceConfigInventory[hostId] ?? browserServiceConfigInventory['browser-host']).find(
        (entry) => entry.serviceName === name,
      );
      const service = (browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']).find(
        (entry) => entry.name === name,
      );
      const shouldRestart = Boolean(service?.running);
      if (config) {
        if (typeof port === 'number' && Number.isFinite(port)) {
          config.port = port;
        }
        if (config.canEditDataDir) {
          config.dataDir = dataDir;
        }
      }
      if (service) {
        if (typeof port === 'number' && Number.isFinite(port)) {
          service.port = port;
        }
        if (config?.canEditDataDir) {
          service.dataDir = dataDir;
        }
        service.notes = [
          `Preview mode wrote a managed ${name} override block for ${hostId}.`,
          ...service.notes.slice(0, 1),
        ];
      }
      const job = seedJob(`Apply ${name} config on ${hostId}`, undefined, undefined, {
        category: 'service',
        targetName: name,
        outcomeTitle: 'Config staged',
        outcomeDetail: `${name} overrides were written on ${hostId} without restarting the service.`,
        nextStep: `Use Apply + ${service?.running ? 'restart' : 'start'} when the new settings should take effect immediately.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_config_apply_and_restart': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      const port =
        typeof args.port === 'number'
          ? args.port
          : args.port == null || args.port === ''
            ? null
            : Number(args.port);
      const dataDir =
        typeof args.dataDir === 'string' && args.dataDir.trim().length ? args.dataDir.trim() : null;
      const config = (browserServiceConfigInventory[hostId] ?? browserServiceConfigInventory['browser-host']).find(
        (entry) => entry.serviceName === name,
      );
      const service = (browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']).find(
        (entry) => entry.name === name,
      );
      const shouldRestart = Boolean(service?.running);
      if (config) {
        if (typeof port === 'number' && Number.isFinite(port)) {
          config.port = port;
        }
        if (config.canEditDataDir) {
          config.dataDir = dataDir;
        }
      }
      if (service) {
        if (typeof port === 'number' && Number.isFinite(port)) {
          service.port = port;
        }
        if (config?.canEditDataDir) {
          service.dataDir = dataDir;
        }
        const reconcileVerb = shouldRestart ? 'restarted' : 'started';
        if (service.installed) {
          service.running = true;
          service.health = 'good';
        }
        service.notes = [
          `Preview mode wrote a managed ${name} override block and ${reconcileVerb} the service on ${hostId}.`,
          ...service.notes.slice(0, 1),
        ];
      }
      const job = seedJob(
        `Apply ${name} config and ${shouldRestart ? 'restart' : 'start'} service on ${hostId}`,
        undefined,
        undefined,
        {
          category: 'service',
          targetName: name,
          outcomeTitle: 'Applied + reconciled',
          outcomeDetail: `${name} overrides were written on ${hostId}, then the service was ${shouldRestart ? 'restarted' : 'started'}.`,
          nextStep: `Verify that ${name} is reachable on its expected local port.`,
        },
      );
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_backup_create': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      const service = (browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']).find(
        (entry) => entry.name === name,
      );
      if (!service?.dataDir || service.running) {
        const job = seedJob(`Create ${name} snapshot backup`, undefined, undefined, {
          status: 'failed',
          category: 'service',
          targetName: name,
          outcomeTitle: service?.running ? 'Unsupported operation' : 'Path unavailable',
          outcomeDetail: service?.running
            ? `${name} must be stopped before preview mode will create a data-dir snapshot archive.`
            : `Preview mode could not determine a data directory for ${name}.`,
          nextStep: service?.running
            ? `Stop ${name} first, then retry snapshot backup.`
            : `Use a service with a detected data directory before retrying backup.`,
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      const archive = {
        serviceName: name,
        kind: 'backup' as const,
        path: `${service.dataDir}/../.forge-env-preview/${name.toLowerCase()}-backup-${Date.now()}.tar.gz`,
        sizeBytes: 131072,
        createdAt: `${Math.floor(Date.now() / 1000)}`,
        managed: true,
      };
      const current = browserServiceArtifactInventory[hostId] ?? [];
      browserServiceArtifactInventory[hostId] = [archive, ...current].slice(0, 8);
      const job = seedJob(`Create ${name} snapshot backup`, undefined, undefined, {
        category: 'service',
        targetName: name,
        outcomeTitle: 'Snapshot backup created',
        outcomeDetail: `${name} preview data-dir snapshot was archived to ${archive.path}.`,
        nextStep: 'Keep the managed archive path if you want to restore or export it later.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_data_export': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      const service = (browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']).find(
        (entry) => entry.name === name,
      );
      if (!service?.dataDir || service.running) {
        const job = seedJob(`Export ${name} data directory`, undefined, undefined, {
          status: 'failed',
          category: 'service',
          targetName: name,
          outcomeTitle: service?.running ? 'Unsupported operation' : 'Path unavailable',
          outcomeDetail: service?.running
            ? `${name} must be stopped before preview mode will export its data-dir snapshot archive.`
            : `Preview mode could not determine a data directory for ${name}.`,
          nextStep: service?.running
            ? `Stop ${name} first, then retry export.`
            : `Use a service with a detected data directory before retrying export.`,
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      const archive = {
        serviceName: name,
        kind: 'export' as const,
        path: `${service.dataDir}/../.forge-env-preview/${name.toLowerCase()}-export-${Date.now()}.tar.gz`,
        sizeBytes: 262144,
        createdAt: `${Math.floor(Date.now() / 1000)}`,
        managed: true,
      };
      const current = browserServiceArtifactInventory[hostId] ?? [];
      browserServiceArtifactInventory[hostId] = [archive, ...current].slice(0, 8);
      const job = seedJob(`Export ${name} data directory`, undefined, undefined, {
        category: 'service',
        targetName: name,
        outcomeTitle: 'Data dir exported',
        outcomeDetail: `${name} preview data-dir contents were exported to ${archive.path}.`,
        nextStep: 'Use the managed export path outside Forge Env or keep it for later restore.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_backup_restore': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      const service = (browserServiceInventory[hostId] ?? browserServiceInventory['browser-host']).find(
        (entry) => entry.name === name,
      );
      const archivePath = typeof args.archivePath === 'string' ? args.archivePath.trim() : '';
      const fallback = (browserServiceArtifactInventory[hostId] ?? []).find(
        (entry) => entry.serviceName === name && entry.kind === 'backup',
      );
      if (service?.running || (!archivePath && !fallback)) {
        const job = seedJob(`Restore ${name} snapshot backup`, undefined, undefined, {
          status: 'failed',
          category: 'service',
          targetName: name,
          outcomeTitle: service?.running ? 'Unsupported operation' : 'Path unavailable',
          outcomeDetail: service?.running
            ? `${name} must be stopped before preview mode will restore a snapshot archive.`
            : `Preview mode has no managed ${name} backup archive available to restore.`,
          nextStep: service?.running
            ? `Stop ${name} first, then retry restore.`
            : `Create a managed snapshot backup first or provide an archive path before retrying restore.`,
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      const chosenPath = archivePath || fallback?.path || '';
      const job = seedJob(`Restore ${name} snapshot backup`, undefined, undefined, {
        category: 'service',
        targetName: name,
        outcomeTitle: 'Snapshot restored',
        outcomeDetail: `${name} preview data-dir contents were restored from ${chosenPath}.`,
        nextStep: `Start ${name} again once the restored snapshot should become live.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_logical_backup_create': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const name = String(args.name ?? '');
      if (name === 'Redis') {
        const job = seedJob(`Create ${name} logical backup`, undefined, undefined, {
          status: 'failed',
          category: 'service',
          targetName: name,
          outcomeTitle: 'Unsupported operation',
          outcomeDetail: 'Preview mode does not expose Redis logical backup yet. Use snapshot backup or export instead.',
          nextStep: 'Use Create backup or Export data dir for Redis until logical backup support exists.',
        });
        browserJobs = [job, ...browserJobs].slice(0, 8);
        return job as T;
      }
      const artifact = {
        serviceName: name,
        kind: 'logical-backup' as const,
        path: `/tmp/${name.toLowerCase()}-logical-${Date.now()}.sql.gz`,
        sizeBytes: 65536,
        createdAt: `${Math.floor(Date.now() / 1000)}`,
        managed: true,
      };
      const current = browserServiceArtifactInventory[hostId] ?? [];
      browserServiceArtifactInventory[hostId] = [artifact, ...current].slice(0, 8);
      const job = seedJob(`Create ${name} logical backup`, undefined, undefined, {
        category: 'service',
        targetName: name,
        outcomeTitle: 'Logical backup created',
        outcomeDetail: `${name} preview logical backup was archived to ${artifact.path}.`,
        nextStep: 'Validate the artifact before depending on it for restore or off-host archive workflows.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_artifact_validate': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const path = String(args.path ?? '');
      const job = seedJob(`Validate service artifact ${path}`, undefined, undefined, {
        category: 'service',
        targetName: path,
        outcomeTitle: 'Artifact validated',
        outcomeDetail: `Preview mode validated the selected artifact path on ${hostId}.`,
        nextStep: 'Keep the artifact if validation passed, or recreate it if you no longer trust the archive.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'service_artifact_delete': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const path = String(args.path ?? '');
      browserServiceArtifactInventory[hostId] = (browserServiceArtifactInventory[hostId] ?? []).filter(
        (artifact) => artifact.path !== path,
      );
      const job = seedJob(`Delete service artifact ${path}`, undefined, undefined, {
        category: 'service',
        targetName: path,
        outcomeTitle: 'Artifact deleted',
        outcomeDetail: `Preview mode removed the selected managed artifact on ${hostId}.`,
        nextStep: 'Refresh service artifacts if you need to confirm the remaining archive inventory.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'mirrors_apply': {
      const mirror = String(args.mirrorSet ?? 'default');
      browserMirrorPreset = mirror;
      const job = seedJob(`Apply mirror preset ${mirror}`, undefined, undefined, {
        category: 'mirror',
        targetName: mirror,
        outcomeTitle: 'Mirror preset applied',
        outcomeDetail: `Mirror preset ${mirror} was applied in preview mode.`,
        nextStep: 'Run a package metadata fetch to confirm the new mirror endpoints resolve correctly.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'env_export': {
      const exportHostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const bundle = {
        fileName: 'forge-env-template.json',
        payload: JSON.stringify(
          {
            schemaVersion: 2,
            hosts: browserHosts,
            hostSnapshots: browserHosts.map((host) => ({
              host,
              runtimes: cloneRuntimeList(host.id),
              systemDependencies: cloneDependencyList(host.id),
            })),
            runtimes: cloneRuntimeList(exportHostId),
            systemDependencies: cloneDependencyList(exportHostId),
            appliedMirrorPreset: browserMirrorPreset,
            generatedAt: new Date().toISOString(),
          },
          null,
          2,
        ),
      };
      return bundle as T;
    }
    case 'env_import': {
      const payload = String(args.payload ?? '{}');
      let parsed: {
        hosts?: Array<{ id?: string; label?: string }>;
        runtimes?: unknown[];
        hostSnapshots?: Array<{
          host?: { id?: string; label?: string };
          runtimes?: Array<{ family?: string; installed?: Array<{ version?: string; active?: boolean }> }>;
          systemDependencies?: Array<{ name?: string; installed?: boolean }>;
        }>;
        appliedMirrorPreset?: string | null;
      } = {};
      const issues: string[] = [];
      try {
        parsed = JSON.parse(payload);
      } catch {
        issues.push('Payload is not valid JSON.');
      }
      const hostSnapshots = Array.isArray(parsed.hostSnapshots) ? parsed.hostSnapshots : [];
      const runtimeCount = Array.isArray(parsed.hostSnapshots) && parsed.hostSnapshots.length
        ? parsed.hostSnapshots.reduce(
            (count, snapshot) => count + (Array.isArray(snapshot.runtimes) ? snapshot.runtimes.length : 0),
            0,
          )
        : Array.isArray(parsed.runtimes)
          ? parsed.runtimes.length
          : 0;
      const result: ImportResult = {
        accepted: issues.length === 0,
        runtimeCount,
        hostCount: Array.isArray(parsed.hostSnapshots) && parsed.hostSnapshots.length
          ? parsed.hostSnapshots.length
          : Array.isArray(parsed.hosts)
            ? parsed.hosts.length
            : 0,
        readyToApply: issues.length === 0 && hostSnapshots.length > 0,
        plannedCount: 0,
        appliedCount: 0,
        issues,
        actions: [],
      };
      if (issues.length === 0) {
        result.actions = hostSnapshots.flatMap((snapshot, hostIndex) => {
          const hostId = String(snapshot.host?.id ?? `host-${hostIndex + 1}`);
          const hostLabel = String(snapshot.host?.label ?? hostId);
          const planned: ImportResult['actions'] = [];
          if (parsed.appliedMirrorPreset) {
            planned.push({
              id: `${hostId}-mirror`,
              hostId,
              hostLabel,
              kind: 'apply-mirror-preset',
              family: null,
              status: 'planned',
              selected: true,
              label: `Apply mirror preset ${parsed.appliedMirrorPreset} on ${hostLabel}`,
              reason: null,
              remediation: null,
              repairAvailable: false,
            });
          }
          const missingDeps = (snapshot.systemDependencies ?? [])
            .filter((dependency) => dependency.installed)
            .map((dependency) => String(dependency.name ?? 'dependency'));
          if (missingDeps.length) {
            planned.push({
              id: `${hostId}-deps`,
              hostId,
              hostLabel,
              kind: 'install-dependencies',
              family: null,
              status: 'planned',
              selected: true,
              label: `Install missing base dependencies on ${hostLabel}: ${missingDeps.join(', ')}`,
              reason: null,
              remediation: null,
              repairAvailable: false,
            });
          }
          (snapshot.runtimes ?? []).forEach((runtime, runtimeIndex) => {
            const family = String(runtime.family ?? `Runtime ${runtimeIndex + 1}`);
            (runtime.installed ?? []).forEach((installation, installationIndex) => {
              const version = String(installation.version ?? 'unknown');
              planned.push({
                id: `${hostId}-${family}-install-${installationIndex}`,
                hostId,
                hostLabel,
                kind: 'install-runtime',
                family,
                status: 'planned',
                selected: true,
                label: `Install ${family} ${version} on ${hostLabel}`,
                reason: null,
                remediation: null,
                repairAvailable: false,
              });
              if (installation.active) {
                planned.push({
                  id: `${hostId}-${family}-activate-${installationIndex}`,
                  hostId,
                  hostLabel,
                  kind: 'activate-runtime',
                  family,
                  status: 'planned',
                  selected: true,
                  label: `Activate ${family} ${version} on ${hostLabel}`,
                  reason: null,
                  remediation: null,
                  repairAvailable: false,
                });
              }
            });
          });
          return planned;
        });
        result.plannedCount = result.actions.filter((action) => action.status === 'planned').length;
      }
      return result as T;
    }
    case 'env_import_apply': {
      const plan = await browserInvoke<ImportResult>('env_import', args);
      const selected = new Set(
        Array.isArray(args.selectedActionIds) ? args.selectedActionIds.map((value) => String(value)) : [],
      );
      const shouldFilter = selected.size > 0;
      const selectedActions = plan.actions.filter(
        (action) => action.status === 'planned' && (!shouldFilter || selected.has(action.id)),
      );
      const applied = {
        ...plan,
        readyToApply: plan.actions.some(
          (action) => action.status === 'planned' && shouldFilter && !selected.has(action.id),
        ),
        accepted: true,
        appliedCount: selectedActions.length,
        plannedCount: plan.actions.filter(
          (action) => action.status === 'planned' && (shouldFilter ? !selected.has(action.id) : false),
        ).length,
        actions: plan.actions.map((action) => ({
          ...action,
          status:
            action.status === 'planned' && (!shouldFilter || selected.has(action.id))
              ? 'applied'
              : action.status,
          selected:
            action.status === 'planned' && (shouldFilter ? !selected.has(action.id) : false),
          reason:
            action.status === 'planned' && (!shouldFilter || selected.has(action.id))
              ? `Preview mode applied ${action.kind} on ${action.hostLabel}.`
              : action.reason,
        })),
      };
      browserJobs = [
        seedJob('Apply imported environment reconstruction plan', undefined, undefined, {
          category: 'import',
          targetName: 'bundle',
          outcomeTitle: 'Import action applied',
          outcomeDetail: `Preview mode applied ${selectedActions.length} planned import action(s).`,
          nextStep: 'Review any remaining planned or blocked actions before considering the preview host reconstructed.',
        }),
        ...browserJobs,
      ].slice(0, 8);
      return applied as T;
    }
    case 'provider_bootstrap': {
      const family = String(args.family ?? 'runtime');
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const runtime = (browserRuntimeInventory[hostId] ?? browserRuntimeInventory['browser-host']).find(
        (entry) => entry.family === family,
      );
      if (runtime) {
        runtime.providerStatus = 'ready';
        runtime.health = runtime.installed.length ? runtime.health : 'attention';
        runtime.capabilities.canInstall = true;
        runtime.capabilities.canActivate = true;
      }
      const job = seedJob(`Bootstrap canonical provider for ${family} on ${hostId}`, family, undefined, {
        category: 'provider',
        targetName: family,
        outcomeTitle: 'Provider bootstrapped',
        outcomeDetail: `The canonical ${family} provider was bootstrapped in preview mode on ${hostId}.`,
        nextStep: `Retry the blocked ${family} action now that its provider path should exist.`,
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'app_preferences':
      return {
        appliedMirrorPreset: browserMirrorPreset,
        selectedHostId: browserSelectedHostId,
        lastEnvTargetProfile: cloneEnvPlan(browserSelectedHostId ?? browserHosts[0].id).targetProfile,
      } as T;
    case 'env_preview':
      return cloneEnvPlan(String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id)) as T;
    case 'env_apply': {
      const hostId = String(args.hostId ?? browserSelectedHostId ?? browserHosts[0].id);
      const profile = String(args.profilePath ?? cloneEnvPlan(hostId).targetProfile);
      const job = seedJob(`Apply Forge Env shell block to ${profile}`, undefined, undefined, {
        category: 'environment',
        targetName: profile,
        outcomeTitle: 'Environment block applied',
        outcomeDetail: `Forge Env wrote the managed shell block to ${profile} in preview mode.`,
        nextStep: 'Open a new shell or source the updated profile so PATH and runtime variables take effect.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'proxy_settings_load':
      return browserProxySettings as T;
    case 'proxy_settings_save': {
      const next = (args.settings ?? browserProxySettings) as ProxySettings;
      browserProxySettings = {
        ...next,
        passwordSaved:
          browserProxySettings.passwordSaved || Boolean(String(args.password ?? '').trim().length),
      };
      const job = seedJob('Save proxy settings', undefined, undefined, {
        category: 'security',
        targetName: 'proxy',
        outcomeTitle: 'Proxy profile saved',
        outcomeDetail: `Preview mode stored the proxy profile for ${next.host || 'the managed proxy target'} and marked the system credential as ${browserProxySettings.passwordSaved ? 'present' : 'absent'}.`,
        nextStep: 'Apply the company proxy preset when this profile should influence mirror and network traffic.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    case 'proxy_settings_clear': {
      browserProxySettings = {
        enabled: false,
        scheme: 'http',
        host: '',
        port: '',
        username: '',
        passwordSaved: false,
        secureStore: 'Keychain',
      };
      const job = seedJob('Clear proxy settings', undefined, undefined, {
        category: 'security',
        targetName: 'proxy',
        outcomeTitle: 'Proxy profile cleared',
        outcomeDetail: 'Preview mode removed the saved proxy profile and cleared the managed credential marker.',
        nextStep: 'Re-enter proxy settings only when the host should route traffic through a managed proxy again.',
      });
      browserJobs = [job, ...browserJobs].slice(0, 8);
      return job as T;
    }
    default:
      throw new Error(`Unsupported browser fallback command: ${command}`);
  }
}

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return browserInvoke<T>(name, args);
  }

  return invoke<T>(name, args);
}

export const api = {
  hostsList: () => command<HostSummary[]>('hosts_list'),
  hostInspect: (hostId: string) => command<HostDetail>('hosts_inspect', { hostId }),
  projectsInspect: (path?: string) => command<ProjectProfile[]>('projects_inspect', { path }),
  projectRuntimeApply: (
    hostId: string,
    projectPath: string,
    family: string,
    version: string,
    options?: ProjectRuntimePolicyOptions,
  ) => command<JobRecord>('project_runtime_apply', { hostId, projectPath, family, version, options }),
  runtimesList: (hostId: string) => command<RuntimeFamilyState[]>('runtimes_list', { hostId }),
  depsList: (hostId: string) => command<SystemDependencyState[]>('deps_list', { hostId }),
  servicesList: (hostId: string) => command<ServiceState[]>('services_list', { hostId }),
  serviceArtifactsList: (hostId: string) =>
    command<ServiceArtifact[]>('service_artifacts_list', { hostId }),
  serviceConfigsList: (hostId: string) =>
    command<ServiceConfigState[]>('services_config_list', { hostId }),
  appPreferences: () => command<AppPreferences>('app_preferences'),
  proxySettingsLoad: () => command<ProxySettings>('proxy_settings_load'),
  runtimesInstall: (hostId: string, family: string, version: string, scope = 'global') =>
    command<JobRecord>('runtimes_install', { hostId, family, version, scope }),
  runtimesSwitch: (hostId: string, family: string, version: string, scope = 'global') =>
    command<JobRecord>('runtimes_switch', { hostId, family, version, scope }),
  runtimesRemove: (hostId: string, family: string, version: string) =>
    command<JobRecord>('runtimes_remove', { hostId, family, version }),
  depsInstall: (hostId: string, dependencies: string[]) => command<JobRecord>('deps_install', { hostId, dependencies }),
  serviceAction: (hostId: string, name: string, action: 'start' | 'stop' | 'restart') =>
    command<JobRecord>('service_action', { hostId, name, action }),
  serviceConfigApply: (
    hostId: string,
    name: string,
    port?: number | null,
    dataDir?: string | null,
  ) => command<JobRecord>('service_config_apply', { hostId, name, port, dataDir }),
  serviceConfigApplyAndRestart: (
    hostId: string,
    name: string,
    port?: number | null,
    dataDir?: string | null,
  ) => command<JobRecord>('service_config_apply_and_restart', { hostId, name, port, dataDir }),
  serviceBackupCreate: (hostId: string, name: string) =>
    command<JobRecord>('service_backup_create', { hostId, name }),
  serviceLogicalBackupCreate: (hostId: string, name: string) =>
    command<JobRecord>('service_logical_backup_create', { hostId, name }),
  serviceDataExport: (hostId: string, name: string) =>
    command<JobRecord>('service_data_export', { hostId, name }),
  serviceBackupRestore: (hostId: string, name: string, archivePath?: string | null) =>
    command<JobRecord>('service_backup_restore', { hostId, name, archivePath }),
  serviceArtifactValidate: (hostId: string, path: string) =>
    command<JobRecord>('service_artifact_validate', { hostId, path }),
  serviceArtifactDelete: (hostId: string, path: string) =>
    command<JobRecord>('service_artifact_delete', { hostId, path }),
  proxySettingsSave: (settings: ProxySettings, password?: string | null) =>
    command<JobRecord>('proxy_settings_save', { settings, password }),
  proxySettingsClear: () => command<JobRecord>('proxy_settings_clear'),
  mirrorsApply: (hostId: string, mirrorSet: string) =>
    command<JobRecord>('mirrors_apply', { hostId, mirrorSet }),
  providerBootstrap: (hostId: string, family: string) =>
    command<JobRecord>('provider_bootstrap', { hostId, family }),
  envExport: (hostId?: string) => command<ExportBundle>('env_export', hostId ? { hostId } : undefined),
  envImport: (payload: string) => command<ImportResult>('env_import', { payload }),
  envImportApply: (payload: string, selectedActionIds?: string[]) =>
    command<ImportResult>('env_import_apply', { payload, selectedActionIds }),
  envPreview: (hostId: string) => command<EnvPlan>('env_preview', { hostId }),
  envApply: (hostId: string, profilePath: string) => command<JobRecord>('env_apply', { hostId, profilePath }),
  jobsSubscribe: () => command<JobRecord[]>('jobs_subscribe'),
};

function describeDotnetPolicyOptions(options?: ProjectRuntimePolicyOptions['dotnet'] | null) {
  if (!options) {
    return '';
  }

  const parts = [
    options.rollForward ? `rollForward=${options.rollForward}` : null,
    typeof options.allowPrerelease === 'boolean'
      ? `allowPrerelease=${String(options.allowPrerelease)}`
      : null,
  ].filter(Boolean);

  return parts.length ? ` (${parts.join(', ')})` : '';
}

function describeCppPolicyOptions(options?: ProjectRuntimePolicyOptions['cpp'] | null) {
  if (!options) {
    return '';
  }

  const parts = [
    options.generator ? `generator=${options.generator}` : null,
    options.binaryDir ? `binaryDir=${options.binaryDir}` : null,
    options.toolchainFile ? `toolchainFile=${options.toolchainFile}` : null,
    options.buildType ? `buildType=${options.buildType}` : null,
  ].filter(Boolean);

  return parts.length ? ` (${parts.join(', ')})` : '';
}
