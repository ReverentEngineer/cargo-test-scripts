//! Main entrypoint
use std::{
    fs::File,
    io::Read,
    process::{
        Command,
        Stdio
    },
    time::{
        Duration,
        Instant
    }
};
use serde::{Deserialize, Serialize};

mod de;

/// Test report information
#[derive(Serialize)]
struct TestReport<'a> {

    /// Name of the test
    #[serde(rename(serialize = "@name"))]
    name: &'a str,

    /// Time elapsed during test
    #[serde(rename(serialize = "@time"))]
    time: f64,

    /// Result of test
    #[serde(rename = "$value")]
    result: Vec<TestResult>
}

/// Suite of tests
#[derive(Serialize)]
#[serde(rename(serialize = "testsuite"))]
struct TestSuiteReport<'a> {
    #[serde(rename = "$value")]
    tests: Vec<TestCase<'a>>
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum TestResult {
    Failure(String)
}

/// A test definition
#[derive(Deserialize)]
struct TestSpec {

    name: String,

    #[serde(default)]
    #[serde(deserialize_with = "de::from_duration_ms")]
    timeout: Option<Duration>,

    script: Vec<String>,

}


fn run_step(command: &str, test_start: &Instant, timeout: &Option<Duration>) -> Result<(), String> {
    let args: Vec<_> = command.split_whitespace().collect();
    let child = Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    match (child, *timeout) {
        (Ok(mut child), Some(timeout)) => {
            while test_start.elapsed() < timeout {
                match child.try_wait() {
                    Ok(None) => (),
                    Ok(Some(status)) if status.success() => return Ok(()),
                    Ok(Some(_)) => {
                        let mut stderr = String::new();
                        child.stderr.expect("Failed to get stderr")
                            .read_to_string(&mut stderr)
                            .expect("Failed to read to string");
                        return Err(stderr.trim_end().to_string())
                    },
                    Err(err) => return Err(format!("{err}"))
                };
            }
            Err(format!("Timed out"))
        },
        (Ok(child), None) => {
            match child.wait_with_output() {
                Ok(output) if output.status.success() => Ok(()),
                Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim_end().to_string()),
                Err(err) => Err(format!("{err}"))
            } 
        },
        (Err(err), _) => Err(format!("{err}")) 
    }
}

impl TestSpec {

    fn run<'a>(&'a self) -> TestReport<'a> {
        let start = Instant::now();
        let mut results= Vec::new();
        for command in &self.script {
            if let Err(err) = run_step(&command, &start, &self.timeout) {
                results.push(TestResult::Failure(err));
                break;
            }
        }
        TestReport {
            name: &self.name,
            time: start.elapsed().as_secs_f64(),
            result: results
        }
    }

}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum TestCase<'a> {
    TestCase(TestReport<'a>)
}

/// Suite of tests
struct TestSuite {
    tests: Vec<TestSpec>
}

impl TestSuite {

    fn run<'a>(&'a self) -> TestSuiteReport<'a> {
        let tests = self.tests.iter()
            .map(|test| TestCase::TestCase(test.run())).collect();
        TestSuiteReport { 
            tests
        }
    }

}


fn main() {
    let matches = clap::Command::new("cargo-test-scripts")
        .bin_name("cargo-test-scripts")
        .arg(clap::Arg::new("manifest").long("manifest-path").default_value("Cargo.toml"))
        .arg(clap::Arg::new("output").long("output").short('o'))
        .get_matches();

    let config = std::fs::read_to_string(matches.get_one::<String>("manifest").unwrap())
        .unwrap_or_else(|err| {
            eprintln!("Unable to read manifest: {err}");
            std::process::exit(-1);
        });

    let test_suite: TestSuite = toml::from_str(&config)
        .unwrap_or_else(|err| {
            eprintln!("Unable to parse tests from Cargo.toml: {err}");
            std::process::exit(-1);
        });

    let report = test_suite.run();

    if let Some(output) = matches.get_one::<String>("output") {
        let output = File::options().create(true).truncate(true).write(true).open(output)
            .unwrap_or_else(|err| {
                eprintln!("Unable to create outputfile: {err}");
                std::process::exit(-1);
            });
        serde_xml_rs::to_writer(output, &report)
    } else {
        serde_xml_rs::to_writer(std::io::stdout(), &report)
    }
    .unwrap_or_else(|err| {
        eprintln!("Unable to write report: {err}");
        std::process::exit(-1);
    });

}
