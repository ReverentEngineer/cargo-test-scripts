//! De-serialization functionality
use std::time::Duration;
use serde::{
    Deserialize,
    de::{
        MapAccess,
        Visitor
    }
};
use crate::{TestSuite, TestSpec};

struct DurationVisitor;
impl<'de> serde::de::Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Duration::from_millis(v as u64))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "duration in milliseconds")
    }
}

pub (crate) fn from_duration_ms<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    d.deserialize_i64(DurationVisitor)
        .map(|d| Some(d))
}

struct Package {
    tests: Vec<TestSpec>
}

impl<'de> Deserialize<'de> for Package {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            struct PackageVisitor;

            impl<'de> Visitor<'de>  for PackageVisitor {
                type Value = Package;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "map")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>
                {
                    while let Some(key) = map.next_key::<String>()? {
                        if key == "metadata" {
                            let Metadata { tests } = map.next_value::<Metadata>()?;
                            return Ok(Package { 
                                tests
                            })
                        } else {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }

                    return Err(serde::de::Error::missing_field("metadata"))
                }
            }

            deserializer.deserialize_map(PackageVisitor)
        }
}

struct Metadata {
    tests: Vec<TestSpec>
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            struct MetadataVisitor;

            impl<'de> Visitor<'de> for MetadataVisitor {
                type Value = Metadata;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "map")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>
                {
                    while let Some(key) = map.next_key::<String>()? {
                        if key == "test-script" {
                            let tests = map.next_value::<Vec<TestSpec>>()?;
                            return Ok(Metadata {
                                tests
                            })
                        } else {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }

                    return Err(serde::de::Error::missing_field("test-script"))
                }
            }
            deserializer.deserialize_map(MetadataVisitor)
        }

}

impl<'de> Deserialize<'de> for TestSuite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            struct CargoConfigVisitor;

            impl<'de> Visitor<'de>  for CargoConfigVisitor {
                type Value = TestSuite;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "map")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>
                {
                    while let Some(key) = map.next_key::<String>()? {
                        if key == "package" {
                            let Package { tests } = map.next_value::<Package>()?;
                            return Ok(TestSuite { 
                                tests
                            })
                        } else {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }

                    return Err(serde::de::Error::missing_field("package"))
                }
            }

            deserializer.deserialize_map(CargoConfigVisitor)
        }
}

