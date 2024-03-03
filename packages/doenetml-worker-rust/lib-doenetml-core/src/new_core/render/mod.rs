//! Functions and objects related to _rendering_ the component tree. A _rendered_ tree is one
//! that will be sent to the UI for display.

pub mod render_children;
pub mod to_flat_dast;

pub use render_children::*;