use std::env;
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode, Stdio};

const WORKSPACE_CARGO_TOML: &str = include_str!("../templates/workspace/Cargo.toml");
const WORKSPACE_MAIN_RS: &str = include_str!("../templates/workspace/src/main.rs");
const PROBLEM_MD: &str = include_str!("../templates/problem/problem.md");
const SAMPLES_README: &str = include_str!("../templates/problem/samples/README.md");
const REFLECTION_MD: &str = include_str!("../templates/reflection.md");

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
    validate_problem_id(problem_id)?;

    let problem_dir = root.join("problems").join(problem_id);
    let learning_dir = root.join("learning").join(problem_id);
    let files = [
        problem_dir.join("Cargo.toml"),
        problem_dir.join("src").join("main.rs"),
        problem_dir.join("problem.md"),
        problem_dir.join("samples").join("README.md"),
        learning_dir.join("reflection.md"),
    ];

    if problem_dir.exists() || learning_dir.exists() || files.iter().any(|path| path.exists()) {
        return Err(HarnessError::new(format!(
            "workspace already exists for problem-id `{problem_id}`"
        )));
    }

    fs::create_dir_all(problem_dir.join("src"))?;
    fs::create_dir_all(problem_dir.join("samples"))?;
    fs::create_dir_all(&learning_dir)?;

    write_new_file(&files[0], WORKSPACE_CARGO_TOML)?;
    write_new_file(&files[1], WORKSPACE_MAIN_RS)?;
    write_new_file(&files[2], &render_template(PROBLEM_MD, problem_id))?;
    write_new_file(&files[3], SAMPLES_README)?;
    write_new_file(&files[4], &render_template(REFLECTION_MD, problem_id))?;

    println!("initialized local workspace: problems/{problem_id}");
    Ok(())
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

    if normalize_output(&output.stdout) != normalize_output(&expected) {
        return Ok(SampleResult::Failed {
            reason: format!(
                "output mismatch (expected `{}`, got `{}`)",
                display_tokens(&normalize_output(&expected)),
                display_tokens(&normalize_output(&output.stdout))
            ),
            stderr: output.stderr,
        });
    }

    Ok(SampleResult::Passed)
}

fn normalize_output(output: &[u8]) -> Vec<&[u8]> {
    output
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
        .collect()
}

fn display_tokens(tokens: &[&[u8]]) -> String {
    tokens
        .iter()
        .map(|token| String::from_utf8_lossy(token).into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{normalize_output, validate_problem_id};

    #[test]
    fn output_comparison_ignores_whitespace() {
        assert_eq!(
            normalize_output(b"1  2\r\n3 \n"),
            normalize_output(b"1\n2 3")
        );
    }

    #[test]
    fn output_comparison_detects_different_tokens() {
        assert_ne!(normalize_output(b"1 2"), normalize_output(b"1 3"));
    }

    #[test]
    fn problem_id_rejects_path_traversal() {
        assert!(validate_problem_id("../outside").is_err());
        assert!(validate_problem_id("a/b").is_err());
        assert!(validate_problem_id("ok_id-1").is_ok());
    }
}
