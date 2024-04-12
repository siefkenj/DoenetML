use std::rc::Rc;

#[cfg(feature = "web")]
use tsify_next::Tsify;
#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

use crate::state::types::math_expr::MathExpr;
use crate::utils::rc_serde;

///////////////////////////////////////////////////////////////////////
// prop enum views that allow one to refer to props
// without specifying type.
// Particularly useful for having vectors of mixed type
///////////////////////////////////////////////////////////////////////

/// The value of a prop tagged with its type.
#[derive(
    Debug,
    Clone,
    PartialEq, // TODO: this might be too much to require on all value types if we have complex types
    serde::Serialize,
    serde::Deserialize,
    derive_more::TryInto,
    derive_more::From,
    doenetml_macros::TryFromRef,
    strum_macros::EnumDiscriminants,
)]
#[serde(untagged)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", tsify(from_wasm_abi))]
pub enum PropValue {
    #[serde(with = "rc_serde")]
    String(prop_type::String),
    Number(prop_type::Number),
    Integer(prop_type::Integer),
    Boolean(prop_type::Boolean),
    #[serde(with = "rc_serde")]
    Math(prop_type::Math),
    // TODO: when create array props, convert this to use the general array mechanism
    // Created a vector type for now.
    #[serde(with = "rc_serde")]
    ComponentRefs(prop_type::ComponentRefs),
    ComponentRef(prop_type::ComponentRef),
    // TODO: when create array props, convert this to use the general array mechanism
    // Created a vector type for now.
    #[serde(with = "rc_serde")]
    ContentRefs(prop_type::ContentRefs),
    ContentRef(prop_type::ContentRef),
}

/// The discriminating type of a `PropValue`.
pub type PropValueType = PropValueDiscriminants;

pub mod prop_type {
    //! Type aliases for the inner type of `PropValue`.
    //! These are are named exactly the same as the discriminants of `PropValue`
    //! so that they can be used in macros.

    use super::*;
    use crate::state::types::{component_refs, content_refs};

    pub type String = Rc<std::string::String>;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type Number = f64;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type Integer = i64;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type Boolean = bool;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type Math = Rc<MathExpr>;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type ComponentRef = Option<component_refs::ComponentRef>;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type ComponentRefs = Rc<component_refs::ComponentRefs>;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type ContentRefs = Rc<content_refs::ContentRefs>;
    #[cfg_attr(feature = "web", tsify_next::declare)]
    pub type ContentRef = content_refs::ContentRef;

    /// By default, wasm-bindgen won't pick up this module as containing types to export
    /// to Typescript. We force wasm-bindgen to export types in this module by providing a
    /// dummy type that is explicitly referenced in `lib-js-wasm-binding/src/lib.rs`.
    #[cfg(feature = "web")]
    #[cfg_attr(
        feature = "web",
        derive(Copy, Clone, Tsify, serde::Serialize, serde::Deserialize)
    )]
    #[cfg_attr(feature = "web", tsify(from_wasm_abi, into_wasm_abi))]
    pub struct _DummyForWasmBindgen {}
}

mod conversions {
    //! Implementation of `From` traits for `PropValue`.

    use super::*;

    impl From<String> for PropValue {
        fn from(v: String) -> Self {
            PropValue::String(Rc::new(v))
        }
    }

    impl From<&str> for PropValue {
        fn from(v: &str) -> Self {
            PropValue::String(Rc::new(v.to_string()))
        }
    }

    impl TryFrom<PropValue> for String {
        type Error = &'static str;
        fn try_from(value: PropValue) -> Result<Self, Self::Error> {
            TryInto::<prop_type::String>::try_into(value).map(|s| (*s).clone())
        }
    }
}