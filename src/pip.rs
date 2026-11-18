use crate::animate::{AnimationSettings, show_loading_indicator};
use crate::cmd::{run, run_get_output};
use anyhow::bail;
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::time::{Duration, timeout};
use uv_pep508::Requirement;

#[derive(Debug, Deserialize)]
struct PipInstallReport {
    install: VecDeque<PipInstallReportItem>,
}

#[derive(Debug, Deserialize)]
struct PipInstallReportItem {
    download_info: PipDownloadReportInfo,
    requested_extras: Option<Vec<String>>,
    metadata: PipMetadataReport,
}

#[derive(Debug, Deserialize)]
struct PipDownloadReportInfo {
    url: String,
}

#[derive(Debug, Deserialize)]
struct PipMetadataReport {
    name: String,
}

pub async fn pip(args: &[&str]) -> anyhow::Result<bool> {
    // unwrap_or_default doesn't work on &&str :(
    let err_prefix = format!("pip {}", args.first().unwrap_or(&""));

    run("pip", args, Some(err_prefix)).await
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct FakeInstallResult {
    pub name: String,
    pub file_url: String,
}

impl FakeInstallResult {
    pub fn to_spec(&self) -> String {
        format!("{} @ {}", self.name, self.file_url)
    }
}

pub async fn fake_install(install_spec: &str) -> anyhow::Result<FakeInstallResult> {
    let tempfile = NamedTempFile::new()?;
    let Some(tempfile_path) = tempfile.as_ref().to_str() else {
        bail!("No temp file could be created for a dry pip install.")
    };

    // array instead of vec:
    let args = [
        "install",
        "--no-deps",
        "--dry-run",
        "--ignore-installed",
        "--ignore-requires-python",
        "--report",
        tempfile_path,
        install_spec,
    ];

    pip(&args).await?;

    let json_file = tempfile.reopen()?;

    let mut pip_data: PipInstallReport = serde_json::from_reader(json_file)?;

    let Some(install) = pip_data.install.pop_front() else {
        bail!("Failed to find package name for local install.")
    };

    // if extras exist, the full name is name[extras]. Otherwise, it's just the name.
    let name = install.requested_extras.as_ref().map_or_else(
        || String::from(&install.metadata.name),
        |extras| format!("{}[{}]", install.metadata.name, extras.join(",")),
    );

    let file_url = install.download_info.url;

    Ok(FakeInstallResult { name, file_url })
}

pub async fn try_parse_local_requirement(
    install_spec: &str
) -> anyhow::Result<(Requirement, String)> {
    // fake install and extract the relevant info
    let promise = fake_install(install_spec);

    let result = show_loading_indicator(
        promise,
        format!("Trying to install local package {install_spec}"),
        AnimationSettings::default(),
    )
    .await?;

    let new_install_spec = result.to_spec();
    let requirement = Requirement::from_str(&new_install_spec)?;

    Ok((requirement, new_install_spec))
}

/// The spec before its `#fragment`, and the fragment with its `#`.
fn split_fragment(install_spec: &str) -> (&str, &str) {
    install_spec
        .find('#')
        .and_then(|index| install_spec.split_at_checked(index))
        .unwrap_or((install_spec, ""))
}

fn normalize_git_source(install_spec: &str) -> Option<String> {
    if Path::new(install_spec).exists() || install_spec.starts_with('.') {
        return None;
    }

    let (source, fragment) = split_fragment(install_spec);

    let Some((scheme, url_rest)) = http_url_parts(source) else {
        if source.contains("://") {
            return None;
        }

        let (repository_path, revision) = source
            .split_once('@')
            .map_or((source, None), |(path, rev)| (path, Some(rev)));
        let (owner, repository_name) = repository_path.split_once('/')?;
        if repository_name.contains('/') || !valid_git_component(owner) {
            return None;
        }

        let repository = repository_name
            .strip_suffix(".git")
            .unwrap_or(repository_name);
        if !valid_git_component(repository)
            || repository == "."
            || repository == ".."
            || revision == Some("")
        {
            return None;
        }

        let at_revision = revision.map_or_else(String::new, |rev| format!("@{rev}"));
        return Some(format!(
            "git+https://github.com/{owner}/{repository}.git{at_revision}{fragment}"
        ));
    };

    let (authority, path_and_query) = url_rest.split_once('/')?;
    if authority.is_empty() || authority.chars().any(char::is_whitespace) {
        return None;
    }

    let (path, _) = path_and_query
        .split_once('?')
        .map_or((path_and_query, None), |(path, query)| (path, Some(query)));
    let path_without_revision = path.split_once('@').map_or(path, |(before, _)| before);
    if path_without_revision.is_empty() {
        return None;
    }

    // Only a path ending in `.git` is taken as a repository without asking it.
    path_without_revision.strip_suffix(".git")?;

    Some(format!(
        "git+{scheme}://{authority}/{path_and_query}{fragment}"
    ))
}

fn http_url_parts(source: &str) -> Option<(&'static str, &str)> {
    source
        .strip_prefix("https://")
        .map(|rest| ("https", rest))
        .or_else(|| source.strip_prefix("http://").map(|rest| ("http", rest)))
}

fn git_probe_url(install_spec: &str) -> Option<String> {
    let source = install_spec
        .split_once('#')
        .map_or(install_spec, |(source, _)| source);
    let (scheme, url_rest) = http_url_parts(source)?;
    let (authority, path_and_query) = url_rest.split_once('/')?;
    if authority.is_empty() || authority.chars().any(char::is_whitespace) {
        return None;
    }

    let (path_with_revision, query) = path_and_query
        .split_once('?')
        .map_or((path_and_query, ""), |(path, rest)| (path, rest));
    let path = path_with_revision
        .split_once('@')
        .map_or(path_with_revision, |(before, _revision)| before);
    if path.is_empty() {
        return None;
    }

    let with_query = if query.is_empty() {
        String::new()
    } else {
        format!("?{query}")
    };

    Some(format!("{scheme}://{authority}/{path}{with_query}"))
}

async fn probe_git_remote(url: &str) -> bool {
    let mut command = Command::new("git");
    command
        .args(["-c", "credential.interactive=never", "ls-remote", url])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0")
        .kill_on_drop(true);

    matches!(
        timeout(Duration::from_secs(5), command.status()).await,
        Ok(Ok(status)) if status.success()
    )
}

async fn resolve_git_source(install_spec: &str) -> Option<String> {
    if let Some(source) = normalize_git_source(install_spec) {
        return Some(source);
    }

    if install_spec.starts_with("git+") {
        return Some(String::from(install_spec));
    }

    let probe_url = git_probe_url(install_spec)?;
    if !probe_git_remote(&probe_url).await {
        return None;
    }

    let (source, fragment) = split_fragment(install_spec);
    Some(format!("git+{source}{fragment}"))
}

fn valid_git_component(component: &str) -> bool {
    !component.is_empty()
        && component
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
}

pub fn is_git_install_spec(install_spec: &str) -> bool {
    let spec = install_spec.trim();
    spec.starts_with("git+")
        || spec
            .split_once('@')
            .is_some_and(|(_, source)| source.trim_start().starts_with("git+"))
        || normalize_git_source(spec).is_some()
}

pub fn sanitize_git_install_spec(install_spec: &str) -> String {
    for scheme in ["git+https://", "git+http://"] {
        let Some((before, after_scheme)) = install_spec.split_once(scheme) else {
            continue;
        };

        let (authority, rest) = after_scheme
            .find('/')
            .and_then(|offset| after_scheme.split_at_checked(offset))
            .unwrap_or((after_scheme, ""));
        let host = authority
            .rsplit_once('@')
            .map_or(authority, |(_, host)| host);

        if host == authority {
            return String::from(install_spec);
        }

        return format!("{before}{scheme}{host}{rest}");
    }

    String::from(install_spec)
}

async fn try_parse_git_requirement(
    install_spec: &str,
    git_source: &str,
) -> anyhow::Result<(Requirement, String)> {
    let promise = fake_install(git_source);

    let result = show_loading_indicator(
        promise,
        format!("Finding package name for {install_spec}"),
        AnimationSettings::default(),
    )
    .await?;

    let new_install_spec = format!("{} @ {git_source}", result.name);
    let requirement = Requirement::from_str(&new_install_spec)?;

    Ok((requirement, new_install_spec))
}

pub async fn parse_requirement(install_spec: &str) -> anyhow::Result<(Requirement, String)> {
    if let Ok(requirement) = Requirement::from_str(install_spec) {
        return Ok((requirement, String::from(install_spec)));
    }

    if let Some(git_source) = resolve_git_source(install_spec).await {
        try_parse_git_requirement(install_spec, &git_source).await
    } else {
        try_parse_local_requirement(install_spec).await
    }
}

pub async fn pip_freeze(python: &Path) -> anyhow::Result<String> {
    // uv pip freeze doesn't work for system
    run_get_output(python, &["-m", "pip", "freeze"]).await
}

#[cfg(test)]
mod tests {
    use super::{
        PipInstallReport, git_probe_url, is_git_install_spec, normalize_git_source,
        parse_requirement, resolve_git_source, sanitize_git_install_spec,
    };
    use core::str::FromStr;
    use core::time::Duration;
    use std::net::{TcpListener, TcpStream};
    use std::process::{Child, Command, Stdio};
    use std::thread;
    use uv_pep508::Requirement;

    struct HttpServer(Child);

    impl Drop for HttpServer {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn run_successfully(command: &mut Command) {
        let output = command.output().expect("run fixture command");
        assert!(
            output.status.success(),
            "command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn serve_git_fixture(directory: &std::path::Path) -> (HttpServer, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve local port");
        let port = listener.local_addr().expect("read local port").port();
        drop(listener);

        let child = Command::new("python3")
            .args([
                "-m",
                "http.server",
                &port.to_string(),
                "--bind",
                "127.0.0.1",
                "--directory",
            ])
            .arg(directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env("NO_PROXY", "127.0.0.1,localhost")
            .env("no_proxy", "127.0.0.1,localhost")
            .spawn()
            .expect("start local HTTP server");
        let server = HttpServer(child);

        let started = (0..100).any(|_| {
            let connected = TcpStream::connect(("127.0.0.1", port)).is_ok();
            if !connected {
                thread::sleep(Duration::from_millis(20));
            }
            connected
        });
        assert!(started, "local HTTP server did not start");

        (server, port)
    }

    fn create_bare_repository(directory: &std::path::Path) {
        let bare_repo = directory.join("repo");
        let work_repo = directory.join("work");
        run_successfully(
            Command::new("git")
                .args(["init", "--bare", "--quiet"])
                .arg(&bare_repo),
        );
        run_successfully(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&work_repo),
        );
        std::fs::write(work_repo.join("README.md"), "fixture\n").expect("write fixture file");
        std::fs::write(
            work_repo.join("setup.py"),
            "from setuptools import setup\nsetup(name='fixture-package', version='1.0.0')\n",
        )
        .expect("write package metadata");
        run_successfully(Command::new("git").arg("-C").arg(&work_repo).args([
            "add",
            "README.md",
            "setup.py",
        ]));
        run_successfully(Command::new("git").arg("-C").arg(&work_repo).args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-q",
            "-m",
            "fixture",
        ]));
        run_successfully(
            Command::new("git")
                .arg("-C")
                .arg(&work_repo)
                .args(["branch", "-M", "main"]),
        );
        run_successfully(
            Command::new("git")
                .arg("-C")
                .arg(&work_repo)
                .arg("push")
                .arg("-q")
                .arg(&bare_repo)
                .arg("main"),
        );
        run_successfully(
            Command::new("git")
                .arg("--git-dir")
                .arg(&bare_repo)
                .args(["update-server-info"]),
        );
    }

    #[test]
    fn normalize_github_shorthand_and_defer_repository_urls_to_probe() {
        assert_eq!(
            normalize_git_source("robinvandernoord/tileception-odoo"),
            Some(String::from(
                "git+https://github.com/robinvandernoord/tileception-odoo.git"
            ))
        );
        assert_eq!(
            normalize_git_source("https://github.com/robinvandernoord/tileception-odoo"),
            None
        );
    }

    #[test]
    fn pip_report_discovery_fields_accept_vcs_local_and_archive_downloads() {
        let report = serde_json::json!({
            "install": [
                {
                    "download_info": {
                        "url": "git+https://example.com/team/project.git",
                        "vcs_info": { "vcs": "git", "commit_id": "abc123" }
                    },
                    "metadata": { "name": "git-package" }
                },
                {
                    "download_info": {
                        "url": "file:///tmp/project",
                        "dir_info": { "editable": false }
                    },
                    "metadata": { "name": "local-package" }
                },
                {
                    "download_info": {
                        "url": "https://example.com/project.tar.gz",
                        "archive_info": { "hashes": {} }
                    },
                    "metadata": { "name": "archive-package" }
                }
            ]
        });

        let parsed = serde_json::from_value::<PipInstallReport>(report)
            .expect("report discovery fields should deserialize");
        let [git, local, archive] =
            <[_; 3]>::try_from(Vec::from(parsed.install)).expect("the report lists three installs");
        assert_eq!(git.metadata.name, "git-package");
        assert_eq!(
            git.download_info.url,
            "git+https://example.com/team/project.git"
        );
        assert_eq!(git.requested_extras, None);
        assert_eq!(local.metadata.name, "local-package");
        assert_eq!(archive.metadata.name, "archive-package");
    }

    #[test]
    fn remove_inline_http_git_credentials_but_preserve_source_details() {
        assert_eq!(
            sanitize_git_install_spec(
                "package[extra] @ git+https://user:token@git.example.com/team/repo.git@feature#subdirectory=app"
            ),
            "package[extra] @ git+https://git.example.com/team/repo.git@feature#subdirectory=app"
        );
        assert_eq!(
            sanitize_git_install_spec(
                "git+http://token@git.example.com/team/repo.git?ref=main#subdirectory=app"
            ),
            "git+http://git.example.com/team/repo.git?ref=main#subdirectory=app"
        );
        assert_eq!(
            sanitize_git_install_spec("git+ssh://user@git.example.com/team/repo.git"),
            "git+ssh://user@git.example.com/team/repo.git"
        );
        assert_eq!(sanitize_git_install_spec("requests>=2"), "requests>=2");
    }

    #[test]
    fn defer_gitlab_and_nested_repository_urls_to_probe() {
        assert_eq!(normalize_git_source("https://gitlab.com/group/repo"), None);
        assert!(!is_git_install_spec("https://gitlab.com/group/repo"));
        assert_eq!(
            normalize_git_source("https://gitlab.com/group/subgroup/repo"),
            None
        );
    }

    #[test]
    fn normalize_git_urls_with_dot_git_on_other_hosts() {
        assert_eq!(
            normalize_git_source("https://git.example.com/team/project.git"),
            Some(String::from("git+https://git.example.com/team/project.git"))
        );
        assert!(is_git_install_spec(
            "https://git.example.com/team/project.git"
        ));
        assert_eq!(
            normalize_git_source("http://git.example.com/team/project.git@v2#subdirectory=python"),
            Some(String::from(
                "git+http://git.example.com/team/project.git@v2#subdirectory=python"
            ))
        );
    }

    #[test]
    fn preserve_git_revision_and_url_fragment() {
        assert_eq!(
            normalize_git_source("owner/repo@v1.2.3#subdirectory=python"),
            Some(String::from(
                "git+https://github.com/owner/repo.git@v1.2.3#subdirectory=python"
            ))
        );
        assert_eq!(
            normalize_git_source("https://github.com/owner/repo.git@release#egg=package"),
            Some(String::from(
                "git+https://github.com/owner/repo.git@release#egg=package"
            ))
        );
        assert_eq!(
            normalize_git_source("owner/repo@feature/branch"),
            Some(String::from(
                "git+https://github.com/owner/repo.git@feature/branch"
            ))
        );
    }

    #[test]
    fn git_probe_strips_revision_and_fragment_but_keeps_query() {
        assert_eq!(
            git_probe_url(
                "https://git.example.com/group/repo@feature/branch?token=secret#subdirectory=app"
            ),
            Some(String::from(
                "https://git.example.com/group/repo?token=secret"
            ))
        );
    }

    #[test]
    fn leave_local_paths_archives_and_other_requirements_unchanged() {
        let temp_dir = tempfile::tempdir().expect("temporary directory");
        let local_path = temp_dir.path().join("owner");
        std::fs::create_dir_all(&local_path).expect("create local path");

        assert_eq!(
            normalize_git_source(local_path.to_str().expect("UTF-8 path")),
            None
        );
        assert_eq!(normalize_git_source("./owner/repo"), None);
        assert_eq!(
            normalize_git_source("https://github.com/owner/repo/archive/main.zip"),
            None
        );
        assert_eq!(
            normalize_git_source("https://gitlab.com/group/repo/-/archive/main.zip"),
            None
        );
        assert_eq!(
            normalize_git_source("https://example.com/download.zip"),
            None
        );
        assert_eq!(normalize_git_source("requests>=2"), None);
        assert!(!is_git_install_spec("requests>=2"));
        assert!(is_git_install_spec("git+https://github.com/owner/repo.git"));
    }

    #[test]
    fn existing_owner_repo_path_is_not_treated_as_github_shorthand() {
        let temp_dir = tempfile::tempdir().expect("temporary directory");
        let local_path = temp_dir.path().join("owner").join("repo");
        std::fs::create_dir_all(&local_path).expect("create local repository directory");

        assert_eq!(
            normalize_git_source(local_path.to_str().expect("UTF-8 path")),
            None
        );
    }

    #[tokio::test]
    async fn standard_pep508_requirements_are_not_rewritten() {
        let spec = "requests>=2";
        let (requirement, resolved) = parse_requirement(spec).await.expect("parse requirement");

        assert_eq!(resolved, spec);
        assert_eq!(
            requirement,
            Requirement::from_str(spec).expect("valid requirement")
        );
    }

    #[tokio::test]
    async fn named_git_requirements_are_preserved() {
        let spec = "tileception-odoo @ git+https://github.com/robinvandernoord/tileception-odoo.git@main#subdirectory=plugin";
        let (requirement, resolved) = parse_requirement(spec).await.expect("parse requirement");

        assert_eq!(resolved, spec);
        assert_eq!(
            requirement,
            Requirement::from_str(spec).expect("valid direct requirement")
        );
        assert!(is_git_install_spec(spec));
        assert!(is_git_install_spec(
            "tileception-odoo@git+https://github.com/owner/repo.git"
        ));
    }

    #[tokio::test]
    async fn probes_arbitrary_http_git_hosts_and_preserves_revision_and_fragment() {
        let temp_dir = tempfile::tempdir().expect("temporary directory");
        create_bare_repository(temp_dir.path());
        let (_server, port) = serve_git_fixture(temp_dir.path());
        let url = format!("http://127.0.0.1:{port}/repo@main#subdirectory=plugin");

        assert_eq!(resolve_git_source(&url).await, Some(format!("git+{url}")));
    }

    #[tokio::test]
    async fn discovers_distribution_name_from_a_local_http_git_repository() {
        let temp_dir = tempfile::tempdir().expect("temporary directory");
        create_bare_repository(temp_dir.path());
        let (_server, port) = serve_git_fixture(temp_dir.path());
        let url = format!("http://127.0.0.1:{port}/repo@main");

        let (requirement, install_spec) = parse_requirement(&url)
            .await
            .expect("discover package from Git pip report");

        assert_eq!(requirement.name.as_ref(), "fixture-package");
        assert!(install_spec.starts_with("fixture-package @ git+http://"));
    }

    #[tokio::test]
    async fn ordinary_http_download_is_not_classified_as_git() {
        let temp_dir = tempfile::tempdir().expect("temporary directory");
        std::fs::write(temp_dir.path().join("download.zip"), "not a repository")
            .expect("write download fixture");
        let (_server, port) = serve_git_fixture(temp_dir.path());
        let url = format!("http://127.0.0.1:{port}/download.zip");

        assert_eq!(resolve_git_source(&url).await, None);
    }
}
