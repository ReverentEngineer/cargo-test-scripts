use crate::{
    Error,
    TestReport,
    TestSuiteReport
};
use chrono::prelude::{DateTime, Utc};

use serde::{
    Serialize,
    ser::SerializeStruct,
    ser::SerializeStructVariant
};

impl Serialize for Error {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            Self::Failure(message) => {
                let mut state = serializer.serialize_struct_variant("error", 0, "failure", 2)?;
                state.serialize_field("@type", "error_code")?;
                state.serialize_field("$value", &message)?;
                state.end()
            },
            Self::Error(message) => {
                let mut state = serializer.serialize_struct_variant("error", 1, "error", 2)?;
                state.serialize_field("@type", "script_error")?;
                state.serialize_field("$value", &message)?;
                state.end()
            }
        }
    }

}

impl<'ser> Serialize for TestReport<'ser> {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut state = serializer.serialize_struct("testcase", 7)?;
        state.serialize_field("@name", &self.name)?;
        state.serialize_field("@classname", "")?;
        state.serialize_field("@time", &self.time.as_secs_f64())?;
        state.serialize_field("$value", &self.result)?;
        state.end()
    }

}

impl<'ser> Serialize for TestSuiteReport<'ser> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut state = serializer.serialize_struct("testsuite", 3)?;
        state.serialize_field("@name", "cargo-test-scripts")?;
        let dt: DateTime<Utc> = self.timestamp.into();
        state.serialize_field("@timestamp", &dt.format("%Y-%m-%dT%H:%M:%S").to_string())?;
        state.serialize_field("@tests", &self.tests())?;
        state.serialize_field("@time", &self.time.as_secs_f64())?;
        state.serialize_field("@failures", &self.failures())?;
        state.serialize_field("@errors", &self.errors())?;
        state.serialize_field("@hostname",
            &gethostname::gethostname().to_str().expect("Failed to get hostname"))?;
        state.serialize_field("$value", &self.contents)?;
        state.end()
    }
}
