use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use crate::errors::Status;

pub(crate) struct SpringResponse<T> {
    pub success: bool,
    pub code: String,
    pub message: String,
    pub data: T,
}

impl<T> SpringResponse<T> {
    pub fn new(success: bool, code: String, message: String, data: T) -> Self {
        Self {
            success,
            code,
            message,
            data,
        }
    }
}

impl<T> From<Status> for SpringResponse<T>
    where T: Default
{
    fn from(value: Status) -> Self {
        Self {
            success: false,
            code: value.reason,
            message: value.message,
            data: T::default(),
        }
    }
}

impl<T> Serialize for SpringResponse<T>
    where T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("SpringResponse", 4)?;
        state.serialize_field("success", &self.success)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("data", &self.data)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for SpringResponse<T>
    where T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }
}

// struct SpringResponseVisitor;
//
// impl<'de, T> Visitor<'de> for SpringResponseVisitor {
//     type Value = SpringResponse<T>;
//
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("struct Duration")
//     }
//
//     fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
//         where
//             V: SeqAccess<'de>,
//     {
//         let secs = seq.next_element()?
//             .ok_or_else(|| de::Error::invalid_length(0, &self))?;
//         let nanos = seq.next_element()?
//             .ok_or_else(|| de::Error::invalid_length(1, &self))?;
//         Ok(SpringResponse::new(secs, nanos))
//     }
//
//     fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
//         where
//             V: MapAccess<'de>,
//     {
//         let mut secs = None;
//         let mut nanos = None;
//         while let Some(key) = map.next_key()? {
//             match key {
//                 Field::Secs => {
//                     if secs.is_some() {
//                         return Err(de::Error::duplicate_field("secs"));
//                     }
//                     secs = Some(map.next_value()?);
//                 }
//                 Field::Nanos => {
//                     if nanos.is_some() {
//                         return Err(de::Error::duplicate_field("nanos"));
//                     }
//                     nanos = Some(map.next_value()?);
//                 }
//             }
//         }
//         let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
//         let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
//         Ok(Duration::new(secs, nanos))
//     }
// }
//
// const FIELDS: &[&str] = &["secs", "nanos"];
// deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seri() {
        let a = SpringResponse::new(false, "OK".to_string(), "OK".to_string(), "OK");
        let b = serde_json::to_string(&a).unwrap();
        println!(" {}", b);
    }
}