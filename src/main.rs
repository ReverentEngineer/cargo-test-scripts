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
        Instant,
        SystemTime
    }
};
use serde::{Deserialize, Serialize};

mod ser;
mod de;

/// Test report information
struct TestReport<'a> {

    /// Name of the test
    name: &'a str,

    /// Time elapsed during test
    time: Duration,

    /// Result of test
    result: Option<Error>
}

impl<'a> TestReport<'a> {

    fn failed(&self) -> bool {
        match self.result {
            Some(Error::Failure(_)) => true,
            _ => false
        }
    }

    fn error(&self) -> bool {
        match self.result {
            Some(Error::Error(_)) => true,
            _ => false
        }
    }
}

/// Suite of tests
struct TestSuiteReport<'a> {

    timestamp: SystemTime, 

    time: Duration,

    contents: Vec<TestSuiteContent<'a>>

}

impl<'a> TestSuiteReport<'a> {

    fn tests(&self) -> usize {
        self.contents.iter().filter(|&test| {
            match test {
                TestSuiteContent::Testcase(_report) => true,
                _ => false
            }
        }).count()
    }

    fn failures(&self) -> usize {
        self.contents.iter().filter(|&test| {
            match test {
                TestSuiteContent::Testcase(report) if report.failed() => true,
                _ => false
            }
        }).count()
    }

    fn errors(&self) -> usize {
        self.contents.iter().filter(|&test| {
            match test {
                TestSuiteContent::Testcase(report) if report.error() => true,
                _ => false
            }
        }).count()
    }
}

enum Error {
    Failure(String),
    Error(String)
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Error(format!("{err}").trim_end().to_string())
    }
}

impl From<std::process::ChildStderr> for Error {
    fn from(mut stderr: std::process::ChildStderr) -> Self {
        let mut message = String::new();
        if let Err(message) = stderr.read_to_string(&mut message) {
            Self::Error(format!("{message}").trim_end().to_string())
        } else {
            Self::Error(message)
        }
    }
}

fn run_step(command: &str, test_start: &Instant, timeout: &Option<Duration>) -> Result<(), Error> {
    let args: Vec<_> = command.split_whitespace().collect();
    let mut child = Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    match *timeout {
        Some(timeout) => {
            while test_start.elapsed() < timeout {
                match child.try_wait()? {
                    None => (),
                    Some(status) if status.success() => return Ok(()),
                    Some(_) => return Err(child.stderr.expect("Failed to get stderr").into()),
                };
            }
            Err(Error::Error(format!("Timed out")))
        },
        None => {
            match child.wait_with_output()? {
                output if output.status.success() => Ok(()),
                output => Err(Error::Failure(String::from_utf8_lossy(&output.stderr).trim_end().to_string())),
            } 
        },
    }
}

impl TestSpec {

    fn run<'a>(&'a self) -> TestReport<'a> {
        let start = Instant::now();
        for command in &self.script {
            if let Err(result) = run_step(command, &start, &self.timeout) {
                return TestReport {
                    name: &self.name,
                    time: start.elapsed(),
                    result: Some(result)
                }
            };
        }
        TestReport {
            name: &self.name,
            time: start.elapsed(),
            result: None
        }
    }

}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum TestSuiteContent<'a> {
    Properties,
    Testcase(TestReport<'a>),
    SystemOut(String),
    SystemErr(String)
}

/// Suite of tests
struct TestSuite {
    tests: Vec<TestSpec>
}

impl TestSuite {

    fn run<'a>(&'a self) -> TestSuiteReport<'a> {
        let timestamp = SystemTime::now();
        let start = Instant::now();
        let mut contents = self.tests.iter()
            .map(|test| TestSuiteContent::Testcase(test.run())).collect::<Vec<_>>();
        contents.insert(0, TestSuiteContent::Properties);
        contents.push(TestSuiteContent::SystemOut(String::new()));
        contents.push(TestSuiteContent::SystemErr(String::new()));
        TestSuiteReport {
            timestamp,
            time: start.elapsed(),
            contents
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
