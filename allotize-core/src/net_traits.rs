use crate::com::com_traits::PoolMetadata;
use crdts::{CmRDT, VClock};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub pool: PoolMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedComponent {
    pub clock: VClock<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "Option::default",
    )]
    pub data: Option<String>,
}

impl Default for VersionedComponent {
    fn default() -> Self {
        Self {
            clock: VClock::default(),
            data: None,
        }
    }
}

impl VersionedComponent {
    pub fn new_with_value(value: impl Into<Option<String>>) -> Self {
        Self {
            clock: VClock::default(),
            data: value.into(),
        }
    }

    pub fn apply(&mut self, actor: String) {
        self.clock.apply(self.clock.inc(actor))
    }
}

#[derive(Debug, Clone)]
pub struct JSVal {
    pub v: JsValue,
}

use serde::ser::{SerializeStruct, Serializer};
impl Serialize for JSVal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("JSVal", 1)?;
        s.serialize_field(
            "v",
            &self.v.into_serde::<String>().map_err(|e| {
                serde::ser::Error::custom(format!("JsValue serialization failed: {}", e))
            })?,
        )?;
        s.end()
    }
}

use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

impl<'de> Deserialize<'de> for JSVal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Inner,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`v`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "v" => Ok(Field::Inner),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct JSValVisitor;

        impl<'de> Visitor<'de> for JSValVisitor {
            type Value = JSVal;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct JSVal")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let v: &str = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(JSVal {
                    v: JsValue::from_serde(v).map_err(|_| de::Error::invalid_length(0, &self))?,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut v: Option<&str> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Inner => {
                            if v.is_some() {
                                return Err(de::Error::duplicate_field("inner"));
                            }
                            v = Some(map.next_value()?);
                        }
                    }
                }
                let inner = v.ok_or_else(|| de::Error::missing_field("inner"))?;
                Ok(JSVal {
                    v: JsValue::from_serde(inner)
                        .map_err(|_| de::Error::invalid_length(0, &self))?,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["inner"];
        deserializer.deserialize_struct("JSVal", FIELDS, JSValVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let vcomp = VersionedComponent {
            clock: VClock::default(),
            data: None,
        };

        let as_str = dbg!(serde_json::to_string(&vcomp));
    }
}
