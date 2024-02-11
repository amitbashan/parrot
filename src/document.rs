use jumprope::JumpRope;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Document {
    pub text: JumpRope,
}

impl Document {
    pub fn insert(&mut self, index: usize, text: &str) {
        self.text.insert(index, text)
    }

    pub fn remove(&mut self, range: std::ops::Range<usize>) {
        self.text.remove(range)
    }
}

impl Serialize for Document {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.text.to_string())
    }
}

struct JumpRopeVisitor;

impl<'de> Visitor<'de> for JumpRopeVisitor {
    type Value = JumpRope;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(JumpRope::from(v))
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(JumpRopeVisitor)
            .map(Self::from)
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Self {
            text: JumpRope::from(value),
        }
    }
}

impl From<JumpRope> for Document {
    fn from(value: JumpRope) -> Self {
        Self { text: value }
    }
}
