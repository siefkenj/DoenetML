//! A Prop is a piece of computed data associated with a component. The value of a Prop
//! is lazily computed and can depend on other Props.

use crate::components::{
    prelude::{ComponentIdx, GraphNode, LocalPropIdx},
    types::AttributeName,
    PropProfile,
};

use super::cache::PropWithMeta;

/// Data resulting from a `DataQuery`
#[derive(Debug)]
pub struct DataQueryResult {
    /// The graph `GraphNode::Query` on the dependency graph that specified what data is needed.
    pub graph_node: GraphNode,
    /// The value of the data that was queried for.
    pub values: Vec<PropWithMeta>,
}

/// A vector of `DataQueryResult`s implemented as a wrapped type.
///
/// The type is wrapped so we can re-implement external traits like `std::ops::Index`.
#[derive(Debug)]
pub struct DataQueryResultVec {
    pub vec: Vec<DataQueryResult>,
}

/// A DataQuery is used to make a Dependency based on the input document structure
#[derive(Debug, Clone, Default, PartialEq)]
pub enum DataQuery {
    /// Query for all children that have a prop that matches the prescribed `PropProfile`s.
    /// Returns the matching props
    ChildPropProfile {
        /// The data query will match child components that have at least one of these profiles
        match_profiles: Vec<PropProfile>,
    },
    /// Query for all children with matching `component_type`.
    /// Returns an ElementRefs with the indices of all match children
    ChildElementRefs { component_type: &'static str },
    /// Query for a particular prop of a component
    Prop {
        /// If None, prop is from the component making the query.
        component_idx: Option<ComponentIdx>,

        /// The prop from component_idx or component making the query.
        local_prop_idx: LocalPropIdx,
    },
    /// Query for a prop from a parent
    ParentProp { prop_profile: PropProfile },
    /// Query for all children of an attribute that match the prescribed `PropProfile`
    Attribute {
        /// The name of the attribute whose children will be matched.
        attribute_name: AttributeName,

        /// The data query will match child components that have at least one of these profiles.
        match_profiles: Vec<PropProfile>,
    },
    #[default]
    /// Will be initialized with the default value of this prop
    /// and will accept any change when inverting.
    State,
}