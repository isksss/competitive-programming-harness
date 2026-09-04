use std::env;
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode, Stdio};

const WORKSPACE_CARGO_TOML: &str = include_str!("../templates/workspace/Cargo.toml");
const WORKSPACE_MAIN_RS: &str = include_str!("../templates/workspace/src/main.rs");
const PROBLEM_MD: &str = include_str!("../templates/problem/problem.md");
const SAMPLES_README: &str = include_str!("../templates/problem/samples/README.md");
const REFLECTION_MD: &str = include_str!("../templates/reflection.md");
const SESSION_YAML: &str = include_str!("../templates/learning/session.yaml");

#[derive(Debug)]
enum Command {
    Init { problem_id: String },
    Test { problem_id: String },
}

#[derive(Debug)]
struct SampleCase {
    name: String,
    input_path: PathBuf,
    expected_path: PathBuf,
}

#[derive(Debug)]
enum SampleResult {
    Passed,
    Failed { reason: String, stderr: Vec<u8> },
}

#[derive(Debug)]
struct HarnessError(String);

impl HarnessError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for HarnessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HarnessError {}

impl From<std::io::Error> for HarnessError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, HarnessError> {
    let command = parse_command(env::args().skip(1))?;
    let root = Path::new(".");

    match command {
        Command::Init { problem_id } => {
            init_problem(root, &problem_id)?;
            Ok(0)
        }
        Command::Test { problem_id } => run_samples(root, &problem_id),
    }
}

fn parse_command<I>(mut args: I) -> Result<Command, HarnessError>
where
    I: Iterator<Item = String>,
{
    let action = args
        .next()
        .ok_or_else(|| HarnessError::new("usage: cph <init|test> <problem-id>"))?;
    let problem_id = args
        .next()
        .ok_or_else(|| HarnessError::new("usage: cph <init|test> <problem-id>"))?;

    if args.next().is_some() {
        return Err(HarnessError::new("usage: cph <init|test> <problem-id>"));
    }

    match action.as_str() {
        "init" => Ok(Command::Init { problem_id }),
        "test" => Ok(Command::Test { problem_id }),
        _ => Err(HarnessError::new(
            "unknown command; expected `init` or `test`",
        )),
    }
}

fn validate_problem_id(problem_id: &str) -> Result<(), HarnessError> {
    if problem_id.is_empty()
        || problem_id == "."
        || problem_id == ".."
        || problem_id.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        })
    {
        return Err(HarnessError::new(
            "problem-id must contain only ASCII letters, digits, `_`, or `-`",
        ));
    }

    Ok(())
}

fn init_problem(root: &Path, problem_id: &str) -> Result<(), HarnessError> {
    init_problem_with_writer(root, problem_id, write_new_file)
}

fn init_problem_with_writer<F>(root: &Path, problem_id: &str, writer: F) -> Result<(), HarnessError>
where
    F: FnMut(&Path, &str) -> Result<(), HarnessError>,
{
    init_problem_with_writer_and_renamer(root, problem_id, writer, |source, destination| {
        fs::rename(source, destination)
    })
}

fn init_problem_with_writer_and_renamer<F, R>(
    root: &Path,
    problem_id: &str,
    mut writer: F,
    mut renamer: R,
) -> Result<(), HarnessError>
where
    F: FnMut(&Path, &str) -> Result<(), HarnessError>,
    R: FnMut(&Path, &Path) -> io::Result<()>,
{
    validate_problem_id(problem_id)?;

    let problem_dir = root.join("problems").join(problem_id);
    let learning_dir = root.join("learning").join(problem_id);
    let files = [
        problem_dir.join("Cargo.toml"),
        problem_dir.join("src").join("main.rs"),
        problem_dir.join("problem.md"),
        problem_dir.join("samples").join("README.md"),
        learning_dir.join("reflection.md"),
        learning_dir.join("session.yaml"),
    ];

    let existing_paths = [
        problem_dir.as_path(),
        learning_dir.as_path(),
        files[0].as_path(),
        files[1].as_path(),
        files[2].as_path(),
        files[3].as_path(),
        files[4].as_path(),
        files[5].as_path(),
    ];
    ensure_paths_absent(&existing_paths, problem_id)?;

    let (staging_problem_dir, staging_learning_dir) = create_staging_dirs(root, problem_id)?;
    let staging_files = [
        staging_problem_dir.join("Cargo.toml"),
        staging_problem_dir.join("src").join("main.rs"),
        staging_problem_dir.join("problem.md"),
        staging_problem_dir.join("samples").join("README.md"),
        staging_learning_dir.join("reflection.md"),
        staging_learning_dir.join("session.yaml"),
    ];

    let write_result = (|| {
        fs::create_dir_all(staging_problem_dir.join("src"))?;
        fs::create_dir_all(staging_problem_dir.join("samples"))?;
        fs::create_dir_all(&staging_learning_dir)?;

        writer(&staging_files[0], WORKSPACE_CARGO_TOML)?;
        writer(&staging_files[1], WORKSPACE_MAIN_RS)?;
        writer(&staging_files[2], &render_template(PROBLEM_MD, problem_id))?;
        writer(&staging_files[3], SAMPLES_README)?;
        writer(
            &staging_files[4],
            &render_template(REFLECTION_MD, problem_id),
        )?;
        writer(&staging_files[5], SESSION_YAML)?;
        Ok::<(), HarnessError>(())
    })();
    if let Err(error) = write_result {
        return Err(with_cleanup(
            error,
            &[
                staging_problem_dir.as_path(),
                staging_learning_dir.as_path(),
            ],
        ));
    }

    if let Err(error) = ensure_paths_absent(&existing_paths, problem_id) {
        return Err(with_cleanup(
            error,
            &[
                staging_problem_dir.as_path(),
                staging_learning_dir.as_path(),
            ],
        ));
    }

    if let Err(error) = renamer(&staging_problem_dir, &problem_dir) {
        return Err(with_cleanup(
            error.into(),
            &[
                staging_problem_dir.as_path(),
                staging_learning_dir.as_path(),
            ],
        ));
    }

    let learning_paths = [learning_dir.as_path(), files[4].as_path()];
    if let Err(error) = ensure_paths_absent(&learning_paths, problem_id) {
        return Err(with_cleanup(
            error,
            &[staging_learning_dir.as_path(), problem_dir.as_path()],
        ));
    }

    if let Err(error) = renamer(&staging_learning_dir, &learning_dir) {
        return Err(with_cleanup(
            error.into(),
            &[staging_learning_dir.as_path(), problem_dir.as_path()],
        ));
    }

    println!("initialized local workspace: problems/{problem_id}");
    Ok(())
}

fn ensure_paths_absent(paths: &[&Path], problem_id: &str) -> Result<(), HarnessError> {
    if paths.iter().any(|path| path.exists()) {
        return Err(HarnessError::new(format!(
            "workspace already exists for problem-id `{problem_id}`"
        )));
    }

    Ok(())
}

fn create_staging_dirs(root: &Path, problem_id: &str) -> Result<(PathBuf, PathBuf), HarnessError> {
    let staging_problem_dir = create_staging_dir(&root.join("problems"), problem_id)?;
    let staging_learning_dir = match create_staging_dir(&root.join("learning"), problem_id) {
        Ok(path) => path,
        Err(error) => {
            return Err(with_cleanup(error, &[staging_problem_dir.as_path()]));
        }
    };

    Ok((staging_problem_dir, staging_learning_dir))
}

fn create_staging_dir(parent: &Path, problem_id: &str) -> Result<PathBuf, HarnessError> {
    fs::create_dir_all(parent)?;

    for attempt in 0..1000 {
        let path = parent.join(format!(
            ".cph-init-{problem_id}-{}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }

    Err(HarnessError::new(format!(
        "could not create a unique staging directory under {}",
        parent.display()
    )))
}

fn with_cleanup(error: HarnessError, paths: &[&Path]) -> HarnessError {
    match cleanup_paths(paths) {
        Ok(()) => error,
        Err(cleanup_error) => HarnessError::new(format!("{error}; {cleanup_error}")),
    }
}

fn cleanup_paths(paths: &[&Path]) -> Result<(), HarnessError> {
    let mut errors = Vec::new();

    for path in paths {
        match fs::remove_dir_all(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HarnessError::new(format!(
            "cleanup failed: {}",
            errors.join("; ")
        )))
    }
}

fn write_new_file(path: &Path, contents: &str) -> Result<(), HarnessError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn render_template(template: &str, problem_id: &str) -> String {
    template.replace("{{PROBLEM_ID}}", problem_id)
}

fn run_samples(root: &Path, problem_id: &str) -> Result<u8, HarnessError> {
    validate_problem_id(problem_id)?;

    let problem_dir = fs::canonicalize(root.join("problems").join(problem_id)).map_err(|_| {
        HarnessError::new(format!(
            "local workspace does not exist: problems/{problem_id}"
        ))
    })?;
    let manifest_path = problem_dir.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(HarnessError::new(format!(
            "missing Cargo manifest: {}",
            manifest_path.display()
        )));
    }

    let cases = collect_samples(&problem_dir.join("samples"))?;
    let total = cases.len();
    let mut passed = 0;

    for case in &cases {
        match run_sample(&problem_dir, &manifest_path, case)? {
            SampleResult::Passed => {
                passed += 1;
                println!("[PASS] {}", case.name);
            }
            SampleResult::Failed { reason, stderr } => {
                println!("[FAIL] {}: {reason}", case.name);
                if !stderr.is_empty() {
                    println!("{}", String::from_utf8_lossy(&stderr));
                }
            }
        }
    }

    println!("sample result: {passed}/{total} passed");
    Ok(if passed == total { 0 } else { 1 })
}

fn collect_samples(samples_dir: &Path) -> Result<Vec<SampleCase>, HarnessError> {
    if !samples_dir.is_dir() {
        return Err(HarnessError::new(format!(
            "missing samples directory: {}",
            samples_dir.display()
        )));
    }

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for entry in fs::read_dir(samples_dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }

        match path.extension() {
            Some(extension) if extension == OsStr::new("in") => inputs.push(path),
            Some(extension) if extension == OsStr::new("out") => outputs.push(path),
            _ => {}
        }
    }

    inputs.sort_by_key(|path| path.file_name().map(OsStr::to_os_string));
    outputs.sort_by_key(|path| path.file_name().map(OsStr::to_os_string));

    if inputs.is_empty() {
        return Err(HarnessError::new(format!(
            "no sample input files found in {}",
            samples_dir.display()
        )));
    }

    let mut cases = Vec::with_capacity(inputs.len());
    for input_path in inputs {
        let expected_path = input_path.with_extension("out");
        if !expected_path.is_file() {
            return Err(HarnessError::new(format!(
                "missing expected output for {}",
                input_path.display()
            )));
        }

        let name = input_path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| HarnessError::new("sample filename must be valid UTF-8"))?
            .to_owned();
        cases.push(SampleCase {
            name,
            input_path,
            expected_path,
        });
    }

    for output_path in outputs {
        if !output_path.with_extension("in").is_file() {
            return Err(HarnessError::new(format!(
                "missing sample input for {}",
                output_path.display()
            )));
        }
    }

    Ok(cases)
}

fn run_sample(
    problem_dir: &Path,
    manifest_path: &Path,
    case: &SampleCase,
) -> Result<SampleResult, HarnessError> {
    let input = fs::File::open(&case.input_path)?;
    let expected = fs::read(&case.expected_path)?;
    let output = ProcessCommand::new("cargo")
        .args(["run", "--quiet", "--manifest-path"])
        .arg(manifest_path)
        .current_dir(problem_dir)
        .stdin(Stdio::from(input))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let status = output.status.code().map_or_else(
            || "terminated by signal".to_owned(),
            |code| code.to_string(),
        );
        return Ok(SampleResult::Failed {
            reason: format!("solution exited with status {status}"),
            stderr: output.stderr,
        });
    }

    let expected_lines = normalize_output(&expected);
    let actual_lines = normalize_output(&output.stdout);
    if actual_lines != expected_lines {
        return Ok(SampleResult::Failed {
            reason: format!(
                "output mismatch (expected `{}`, got `{}`)",
                display_output(&expected_lines),
                display_output(&actual_lines)
            ),
            stderr: output.stderr,
        });
    }

    Ok(SampleResult::Passed)
}

fn normalize_output(output: &[u8]) -> Vec<Vec<&[u8]>> {
    let has_final_newline = output.ends_with(b"\n");
    let mut lines = output.split(|byte| *byte == b'\n').collect::<Vec<_>>();
    if has_final_newline {
        lines.pop();
    }

    let line_count = lines.len();
    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let line = if has_final_newline || index + 1 < line_count {
                line.strip_suffix(b"\r").unwrap_or(line)
            } else {
                line
            };

            line.split(|byte| *byte == b' ' || *byte == b'\t')
                .filter(|token| !token.is_empty())
                .collect()
        })
        .collect()
}

fn display_output(lines: &[Vec<&[u8]>]) -> String {
    lines
        .iter()
        .map(|line| {
            let tokens = line
                .iter()
                .map(|token| String::from_utf8_lossy(token).into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            format!("[{tokens}]")
        })
        .collect::<Vec<_>>()
        .join("\\n")
}

#[cfg(test)]
mod tests {
    use super::{
        display_output, init_problem, init_problem_with_writer,
        init_problem_with_writer_and_renamer, normalize_output, validate_problem_id,
        write_new_file, HarnessError,
    };
    use std::fs;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEMP_ROOT: AtomicUsize = AtomicUsize::new(0);

    struct TemporaryRoot {
        path: PathBuf,
    }

    impl TemporaryRoot {
        fn new(name: &str) -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock must be after the Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "competitive-programming-harness-{name}-{}-{timestamp}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("temporary test root should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TemporaryRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn session_path(root: &Path, problem_id: &str) -> PathBuf {
        root.join("learning").join(problem_id).join("session.yaml")
    }

    #[test]
    fn init_creates_initial_hint_session() {
        let root = TemporaryRoot::new("init-session");

        init_problem(root.path(), "abc001_c").expect("initialization should succeed");

        assert_eq!(
            fs::read_to_string(session_path(root.path(), "abc001_c"))
                .expect("session state should be readable"),
            "hint_level: 0\nsolution_revealed: false\n"
        );
    }

    #[test]
    fn init_preserves_existing_workspace_and_hint_session() {
        let root = TemporaryRoot::new("preserve-session");
        let problem_id = "abc001_c";

        init_problem(root.path(), problem_id).expect("initialization should succeed");
        let files = [
            root.path().join("problems/abc001_c/Cargo.toml"),
            root.path().join("problems/abc001_c/src/main.rs"),
            root.path().join("problems/abc001_c/problem.md"),
            root.path().join("problems/abc001_c/samples/README.md"),
            root.path().join("learning/abc001_c/reflection.md"),
            session_path(root.path(), problem_id),
        ];
        let before = files
            .iter()
            .map(|path| fs::read(path).expect("initialized file should be readable"))
            .collect::<Vec<_>>();

        assert!(init_problem(root.path(), problem_id).is_err());

        let after = files
            .iter()
            .map(|path| fs::read(path).expect("existing file should remain readable"))
            .collect::<Vec<_>>();
        assert_eq!(before, after);
    }

    #[test]
    fn hint_session_updates_and_reloads() {
        let root = TemporaryRoot::new("update-session");
        let problem_id = "abc001_c";
        let path = session_path(root.path(), problem_id);

        init_problem(root.path(), problem_id).expect("initialization should succeed");
        assert_eq!(
            fs::read_to_string(&path).expect("initial session state should be readable"),
            "hint_level: 0\nsolution_revealed: false\n"
        );

        fs::write(&path, "hint_level: 1\nsolution_revealed: false\n")
            .expect("level 1 session state should be writable");
        assert_eq!(
            fs::read_to_string(&path).expect("level 1 session state should be readable"),
            "hint_level: 1\nsolution_revealed: false\n"
        );

        fs::write(&path, "hint_level: 2\nsolution_revealed: false\n")
            .expect("level 2 session state should be writable");
        assert_eq!(
            fs::read_to_string(&path).expect("level 2 session state should be readable"),
            "hint_level: 2\nsolution_revealed: false\n"
        );

        fs::write(&path, "hint_level: 6\nsolution_revealed: true\n")
            .expect("revealed session state should be writable");
        assert_eq!(
            fs::read_to_string(&path).expect("revealed session state should be readable"),
            "hint_level: 6\nsolution_revealed: true\n"
        );
    }

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new() -> Self {
            loop {
                let counter = NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "competitive-programming-harness-init-{}-{counter}",
                    std::process::id()
                ));

                match fs::create_dir(&path) {
                    Ok(()) => return Self { path },
                    Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("failed to create test root: {error}"),
                }
            }
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn assert_initialized(root: &Path, problem_id: &str) {
        let files = [
            root.join("problems").join(problem_id).join("Cargo.toml"),
            root.join("problems")
                .join(problem_id)
                .join("src")
                .join("main.rs"),
            root.join("problems").join(problem_id).join("problem.md"),
            root.join("problems")
                .join(problem_id)
                .join("samples")
                .join("README.md"),
            root.join("learning").join(problem_id).join("reflection.md"),
            root.join("learning").join(problem_id).join("session.yaml"),
        ];

        for file in files {
            assert!(
                file.is_file(),
                "missing initialized file: {}",
                file.display()
            );
        }
    }

    fn assert_no_workspace_or_staging(root: &Path, problem_id: &str) {
        assert!(!root.join("problems").join(problem_id).exists());
        assert!(!root.join("learning").join(problem_id).exists());

        let staging_prefix = format!(".cph-init-{problem_id}-");
        for parent_name in ["problems", "learning"] {
            let parent = root.join(parent_name);
            let Ok(entries) = fs::read_dir(parent) else {
                continue;
            };

            for entry in entries {
                let entry = entry.expect("failed to inspect staging directory");
                let name = entry.file_name();
                assert!(
                    !name.to_string_lossy().starts_with(&staging_prefix),
                    "staging directory was not removed: {}",
                    entry.path().display()
                );
            }
        }
    }

    #[test]
    fn output_comparison_accepts_crlf() {
        assert_eq!(normalize_output(b"1 2\r\n3"), normalize_output(b"1 2\n3"));
    }

    #[test]
    fn output_comparison_accepts_trailing_whitespace() {
        assert_eq!(normalize_output(b"1 2  \n3\t"), normalize_output(b"1 2\n3"));
    }

    #[test]
    fn output_comparison_accepts_one_optional_final_newline() {
        assert_eq!(normalize_output(b"1 2\n3\n"), normalize_output(b"1 2\n3"));
        assert_ne!(normalize_output(b"1 2\n3\n\n"), normalize_output(b"1 2\n3"));
    }

    #[test]
    fn output_comparison_accepts_spaces_and_tabs_within_a_line() {
        assert_eq!(
            normalize_output(b" \t1  2\t3 \t"),
            normalize_output(b"1\t2 3")
        );
    }

    #[test]
    fn output_comparison_detects_tokens_on_different_lines() {
        assert_ne!(normalize_output(b"1\n2 3"), normalize_output(b"1 2\n3"));
    }

    #[test]
    fn output_comparison_detects_different_blank_line_positions() {
        assert_ne!(normalize_output(b"1\n\n2"), normalize_output(b"1\n2"));
    }

    #[test]
    fn output_comparison_detects_different_tokens() {
        assert_ne!(normalize_output(b"1 2"), normalize_output(b"1 3"));
    }

    #[test]
    fn output_display_keeps_line_structure() {
        assert_eq!(
            display_output(&normalize_output(b"1\n\n2")),
            "[1]\\n[]\\n[2]"
        );
    }

    #[test]
    fn problem_id_rejects_path_traversal() {
        assert!(validate_problem_id("../outside").is_err());
        assert!(validate_problem_id("a/b").is_err());
        assert!(validate_problem_id("ok_id-1").is_ok());
    }

    #[test]
    fn init_problem_rolls_back_file_creation_failures_and_can_retry() {
        for fail_at in 1_usize..=6 {
            let root = TempRoot::new();
            let mut writes = 0;
            let result = init_problem_with_writer(&root.path, "abc", |path, contents| {
                writes += 1;
                if writes == fail_at {
                    Err(HarnessError::new("injected file creation failure"))
                } else {
                    write_new_file(path, contents)
                }
            });

            assert!(result.is_err(), "file creation failure was not propagated");
            assert_no_workspace_or_staging(&root.path, "abc");

            init_problem(&root.path, "abc").expect("initialization should be retryable");
            assert_initialized(&root.path, "abc");
        }
    }

    #[test]
    fn init_problem_rolls_back_when_either_rename_fails() {
        for fail_at in 1_usize..=2 {
            let root = TempRoot::new();
            let mut renames = 0;
            let result = init_problem_with_writer_and_renamer(
                &root.path,
                "abc",
                write_new_file,
                |source, destination| {
                    renames += 1;
                    if renames == fail_at {
                        Err(std::io::Error::new(
                            ErrorKind::Other,
                            "injected rename failure",
                        ))
                    } else {
                        fs::rename(source, destination)
                    }
                },
            );

            assert!(result.is_err(), "rename failure was not propagated");
            assert_no_workspace_or_staging(&root.path, "abc");

            init_problem(&root.path, "abc").expect("initialization should be retryable");
            assert_initialized(&root.path, "abc");
        }
    }

    #[test]
    fn init_problem_does_not_overwrite_existing_workspace() {
        let root = TempRoot::new();
        init_problem(&root.path, "abc").expect("initialization should succeed");

        let problem_main = root
            .path
            .join("problems")
            .join("abc")
            .join("src")
            .join("main.rs");
        let learning_reflection = root.path.join("learning").join("abc").join("reflection.md");
        let original_problem_main = fs::read(&problem_main).expect("failed to read solution");
        let original_learning_reflection =
            fs::read(&learning_reflection).expect("failed to read reflection");

        let error = init_problem(&root.path, "abc").expect_err("reinitialization should fail");
        assert!(error.to_string().contains("workspace already exists"));
        assert_eq!(
            fs::read(&problem_main).expect("failed to read solution after retry"),
            original_problem_main
        );
        assert_eq!(
            fs::read(&learning_reflection).expect("failed to read reflection after retry"),
            original_learning_reflection
        );
    }
}
