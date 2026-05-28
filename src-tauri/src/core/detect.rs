use std::{fs, path::Path};

use super::models::{ProjectProfile, SuggestedRuntime};

const MARKERS: &[&str] = &[
    ".python-version",
    "pyproject.toml",
    "requirements.txt",
    ".nvmrc",
    "package.json",
    "Cargo.toml",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "go.mod",
    "global.json",
    "composer.json",
    "Gemfile",
    ".ruby-version",
    "CMakeLists.txt",
    "compile_commands.json",
    "Makefile",
];

pub fn scan_projects(root: &Path) -> Vec<ProjectProfile> {
    let mut candidates = vec![root.to_path_buf()];

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                candidates.push(path);
            }
        }
    }

    candidates
        .into_iter()
        .filter_map(|path| detect_project(&path))
        .collect()
}

fn detect_project(path: &Path) -> Option<ProjectProfile> {
    let markers = collect_markers(path);

    if markers.is_empty() {
        return None;
    }

    let mut suggested_runtimes = Vec::new();
    let mut risk_flags = Vec::new();

    if markers
        .iter()
        .any(|marker| marker == "pyproject.toml" || marker == ".python-version")
    {
        suggested_runtimes.push(SuggestedRuntime {
            family: "Python".into(),
            version: "3.12".into(),
            reason: "Python project markers detected.".into(),
        });
    }

    if markers
        .iter()
        .any(|marker| marker == "package.json" || marker == ".nvmrc")
    {
        suggested_runtimes.push(SuggestedRuntime {
            family: "Node.js".into(),
            version: "20 LTS".into(),
            reason: "JavaScript project markers detected.".into(),
        });
    }

    if markers.iter().any(|marker| marker == "Cargo.toml") {
        suggested_runtimes.push(SuggestedRuntime {
            family: "Rust".into(),
            version: "stable".into(),
            reason: "Cargo manifest detected.".into(),
        });
    }

    if markers.iter().any(|marker| {
        marker == "pom.xml" || marker == "build.gradle" || marker == "build.gradle.kts"
    }) {
        suggested_runtimes.push(SuggestedRuntime {
            family: "Java".into(),
            version: "21".into(),
            reason: "Java build markers detected.".into(),
        });
    }

    if markers.iter().any(|marker| marker == "go.mod") {
        suggested_runtimes.push(SuggestedRuntime {
            family: "Go".into(),
            version: "1.22".into(),
            reason: "go.mod detected.".into(),
        });
    }

    if markers.iter().any(|marker| {
        marker == "CMakeLists.txt" || marker == "compile_commands.json" || marker == "Makefile"
    }) {
        suggested_runtimes.push(SuggestedRuntime {
            family: "C/C++".into(),
            version: "system toolchain".into(),
            reason: "Native build markers detected.".into(),
        });
    }

    if markers.iter().any(|marker| marker == "composer.json") {
        suggested_runtimes.push(SuggestedRuntime {
            family: "PHP".into(),
            version: "8.2".into(),
            reason: "composer.json detected.".into(),
        });
    }

    if markers
        .iter()
        .any(|marker| marker == "Gemfile" || marker == ".ruby-version")
    {
        suggested_runtimes.push(SuggestedRuntime {
            family: "Ruby".into(),
            version: "3.2".into(),
            reason: "Ruby project markers detected.".into(),
        });
    }

    if markers.iter().any(|marker| {
        marker == "global.json" || marker == "*.csproj" || marker == "*.fsproj" || marker == "*.sln"
    }) {
        suggested_runtimes.push(SuggestedRuntime {
            family: ".NET".into(),
            version: "8.0 LTS".into(),
            reason: ".NET solution or SDK markers detected.".into(),
        });
    }

    if markers.iter().any(|marker| marker == "requirements.txt")
        && !markers.iter().any(|marker| marker == "pyproject.toml")
    {
        risk_flags.push(
            "requirements.txt present without pyproject.toml; packaging strategy may be split."
                .into(),
        );
    }

    if markers.iter().any(|marker| marker == "package.json")
        && !markers.iter().any(|marker| marker == ".nvmrc")
    {
        risk_flags
            .push("package.json present without .nvmrc; Node version pinning may drift.".into());
    }

    if markers
        .iter()
        .any(|marker| marker == "build.gradle" || marker == "build.gradle.kts")
        && !markers.iter().any(|marker| marker == "pom.xml")
    {
        risk_flags.push(
            "Gradle build detected without pom.xml; ensure Java version policy is captured elsewhere."
                .into(),
        );
    }

    if markers
        .iter()
        .any(|marker| marker == "*.csproj" || marker == "*.fsproj" || marker == "*.sln")
        && !markers.iter().any(|marker| marker == "global.json")
    {
        risk_flags.push(
            ".NET project markers detected without global.json; SDK resolution may drift between hosts."
                .into(),
        );
    }

    if markers.iter().any(|marker| marker == "composer.json")
        && !markers.iter().any(|marker| marker == "package.json")
    {
        risk_flags.push(
            "composer.json detected without adjacent frontend markers; verify whether PHP assets depend on a separate Node toolchain."
                .into(),
        );
    }

    if markers.iter().any(|marker| marker == "Gemfile")
        && !markers.iter().any(|marker| marker == ".ruby-version")
    {
        risk_flags.push(
            "Gemfile detected without .ruby-version; Ruby version pinning may drift between hosts."
                .into(),
        );
    }

    if markers.iter().any(|marker| marker == "Makefile")
        && !markers.iter().any(|marker| marker == "CMakeLists.txt")
    {
        risk_flags.push(
            "Makefile detected without CMakeLists.txt; ensure compiler and flag policy is captured elsewhere."
                .into(),
        );
    }

    Some(ProjectProfile {
        id: format!("project-{}", slugify(path)),
        name: path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "workspace-root".to_string()),
        path: path.to_string_lossy().to_string(),
        markers,
        suggested_runtimes,
        risk_flags: risk_flags.clone(),
        health: if risk_flags.is_empty() {
            "good".into()
        } else {
            "attention".into()
        },
    })
}

fn collect_markers(path: &Path) -> Vec<String> {
    let mut markers = MARKERS
        .iter()
        .filter_map(|marker| {
            let candidate = path.join(marker);
            candidate.exists().then(|| (*marker).to_string())
        })
        .collect::<Vec<_>>();

    if let Ok(entries) = fs::read_dir(path) {
        let mut has_csproj = false;
        let mut has_fsproj = false;
        let mut has_sln = false;

        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                has_csproj |= name.ends_with(".csproj");
                has_fsproj |= name.ends_with(".fsproj");
                has_sln |= name.ends_with(".sln");
            }
        }

        if has_csproj {
            markers.push("*.csproj".into());
        }
        if has_fsproj {
            markers.push("*.fsproj".into());
        }
        if has_sln {
            markers.push("*.sln".into());
        }
    }

    markers
}

fn slugify(path: &Path) -> String {
    path.components()
        .map(|part| {
            part.as_os_str()
                .to_string_lossy()
                .replace(|ch: char| !ch.is_ascii_alphanumeric(), "-")
        })
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        env::temp_dir().join(format!("{prefix}-{unique}"))
    }

    #[test]
    fn detects_python_project_markers() {
        let root = temp_dir("forge-env-python");
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("pyproject.toml"), "[project]\nname='demo'").expect("write pyproject");

        let result = scan_projects(&root);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, root.file_name().unwrap().to_string_lossy());
        assert!(result[0].markers.contains(&"pyproject.toml".to_string()));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_nested_project_markers() {
        let root = temp_dir("forge-env-nested");
        let child = root.join("frontend");
        fs::create_dir_all(&child).expect("create child");
        fs::write(child.join("package.json"), "{}").expect("write package");
        fs::write(child.join(".nvmrc"), "20").expect("write nvmrc");

        let result = scan_projects(&root);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "frontend");
        assert_eq!(result[0].health, "good");

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_java_and_go_project_markers() {
        let root = temp_dir("forge-env-jvm-go");
        let java = root.join("service");
        let go = root.join("cli");
        fs::create_dir_all(&java).expect("create java");
        fs::create_dir_all(&go).expect("create go");
        fs::write(java.join("pom.xml"), "<project/>").expect("write pom");
        fs::write(go.join("go.mod"), "module example.com/cli").expect("write go mod");

        let result = scan_projects(&root);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|project| {
            project
                .suggested_runtimes
                .iter()
                .any(|runtime| runtime.family == "Java")
        }));
        assert!(result.iter().any(|project| {
            project
                .suggested_runtimes
                .iter()
                .any(|runtime| runtime.family == "Go")
        }));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_dotnet_project_markers() {
        let root = temp_dir("forge-env-dotnet");
        let dotnet = root.join("worker");
        fs::create_dir_all(&dotnet).expect("create dotnet");
        fs::write(dotnet.join("worker.csproj"), "<Project />").expect("write csproj");
        fs::write(
            dotnet.join("global.json"),
            "{ \"sdk\": { \"version\": \"8.0.303\" } }",
        )
        .expect("write global json");

        let result = scan_projects(&root);
        assert_eq!(result.len(), 1);
        assert!(result[0].markers.contains(&"*.csproj".to_string()));
        assert!(result[0].markers.contains(&"global.json".to_string()));
        assert!(result[0]
            .suggested_runtimes
            .iter()
            .any(|runtime| runtime.family == ".NET"));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_php_and_ruby_project_markers() {
        let root = temp_dir("forge-env-php-ruby");
        let php = root.join("cms");
        let ruby = root.join("jobs");
        fs::create_dir_all(&php).expect("create php");
        fs::create_dir_all(&ruby).expect("create ruby");
        fs::write(php.join("composer.json"), "{}").expect("write composer");
        fs::write(ruby.join("Gemfile"), "source 'https://rubygems.org'").expect("write gemfile");
        fs::write(ruby.join(".ruby-version"), "3.2.2").expect("write ruby version");

        let result = scan_projects(&root);
        assert!(result.iter().any(|project| {
            project
                .suggested_runtimes
                .iter()
                .any(|runtime| runtime.family == "PHP")
        }));
        assert!(result.iter().any(|project| {
            project
                .suggested_runtimes
                .iter()
                .any(|runtime| runtime.family == "Ruby")
        }));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_cpp_project_markers() {
        let root = temp_dir("forge-env-cpp");
        let native = root.join("native");
        fs::create_dir_all(&native).expect("create native");
        fs::write(
            native.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.25)",
        )
        .expect("write cmake");

        let result = scan_projects(&root);
        assert!(result.iter().any(|project| {
            project
                .suggested_runtimes
                .iter()
                .any(|runtime| runtime.family == "C/C++")
        }));

        fs::remove_dir_all(root).expect("cleanup");
    }
}
