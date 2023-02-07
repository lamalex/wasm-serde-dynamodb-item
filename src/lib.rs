#![warn(clippy::all, clippy::pedantic, clippy::perf)]

use js_sys::JsString;
use serde::de::DeserializeOwned;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod de;
use de::Deserializer;

fn static_str_to_js(s: &'static str) -> JsString {
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct PtrHasher {
        addr: usize,
    }

    impl std::hash::Hasher for PtrHasher {
        fn write(&mut self, _bytes: &[u8]) {
            unreachable!();
        }

        fn write_usize(&mut self, addr_or_len: usize) {
            if self.addr == 0 {
                self.addr = addr_or_len;
            }
        }

        fn finish(&self) -> u64 {
            self.addr as _
        }
    }

    type PtrBuildHasher = std::hash::BuildHasherDefault<PtrHasher>;

    thread_local! {
        // Since we're mainly optimising for converting the exact same string literal over and over again,
        // which will always have the same pointer, we can speed things up by indexing by the string's pointer
        // instead of its value.
        static CACHE: RefCell<HashMap<*const str, JsString, PtrBuildHasher>> = Default::default();
    }
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(s)
            .or_insert_with(|| s.into())
            .clone()
    })
}

/// `from_jsvalue` takes a Json `DynamoDb` item and deserializes
/// it into a type that implementes [`serde::Deserialize`].
/// # Errors
/// - Deser is a wrapper for a variety of errors that can occur during serde type conversion
//    see [`de::Error`] for more information on the inner errors that this type wraps
pub fn from_jsvalue<T>(v: JsValue) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    // this isn't right!!! i need my own impl of From<JsValue>, i think
    // this won't correctly take an attribute value into a native type
    T::deserialize(Deserializer::from(v))
}

/// Deserialize errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// There was an error from a `Deserialize` type
    #[error("error from deserialize type: {0}")]
    Custom(String),
    #[error("deserialization must be into either a struct or a map")]
    UnsupportedType,
    #[error("the attribute value's declared type did not match deserialization target, {0}")]
    UnexpectedType(&'static str),
    #[error("the attribute value's declared type did not match deserialization target, {0:?}")]
    UnexpectedValue(JsValue),
}

#[wasm_bindgen]
extern "C" {
    type ObjectExt;

    #[wasm_bindgen(method, indexing_getter)]
    fn get_with_ref_key(this: &ObjectExt, key: &JsString) -> JsValue;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &ObjectExt, key: JsString, value: JsValue);
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Foo {
        s: String,
        // BigInt is arbitrarily large, so this can overflow.
        // Dynamo doesn't differentiate between Number and BigInt
        n_i128: i128,
        n_i64: i64,
        n_i32: i32,
    }

    #[wasm_bindgen_test]
    fn test_deserialize_into_struct() {
        let json = r#"
        {
            "s": { 
                "S": "Example" 
            },
            "n_i128": {
                "N": "170141183460469231731687303715884105727"
            },
            "n_i64": {
                "N": "9223372036854775807"
            },
            "n_i32": {
                "N": "2147483647"
            }
        }"#;
        let js_value = js_sys::JSON::parse(json).unwrap();

        let expected = Foo {
            s: "Example".into(),
            n_i128: i128::MAX,
            n_i64: i64::MAX,
            n_i32: i32::MAX,
        };

        let actual: Foo = super::from_jsvalue(js_value).unwrap();
        assert_eq!(expected, actual);
    }
}
