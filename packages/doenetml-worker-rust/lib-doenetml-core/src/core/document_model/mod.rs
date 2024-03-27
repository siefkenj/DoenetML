//! # DocumentModel
//!
//! This module contains information about the structure of the document as well as the state of the document.
//! It can be queried for information about the document and its components.

mod dependency_creation;
#[allow(clippy::module_inception)]
mod document_model;
mod prop_calculation;

pub use document_model::*;

#[cfg(test)]
mod debug;