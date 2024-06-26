use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    components::{
        prelude::{ComponentState, PropIdx, PropValue, UntaggedContent},
        ComponentEnum, ComponentNode, ComponentProfile,
    },
    state::prop_state::{create_state_data_for, InitialStateData, StateProp, StatePropDataOrigin},
    ComponentIdx, Extending,
};

use super::{
    dependency_creation_utils::{
        get_attribute_with_parent_falling_back_to_extend_source,
        get_children_with_parent_including_from_extend_source,
        get_same_type_component_extend_source_origin,
    },
    DataQuery, DependenciesCreatedForDataQuery, Dependency, DependencySource,
};

/// Create the dependencies specified by the data query
/// by finding elements in the document that match.
///
/// If an data query asks for state data, create it and add it to *state_data*.
pub fn create_dependencies_from_data_query_initialize_state(
    components: &Vec<Rc<RefCell<ComponentEnum>>>,
    component_idx: ComponentIdx,
    prop_idx: PropIdx,
    query: &DataQuery,
    state_data: &mut Vec<HashMap<StatePropDataOrigin, StateProp>>,
) -> DependenciesCreatedForDataQuery {
    match query {
        DataQuery::State => {
            // We recurse to extend source components so that this state data
            // is shared with the extend source of any other components that is extending from it.
            let source_idx =
                get_same_type_component_extend_source_origin(components, component_idx);

            let state_origin = StatePropDataOrigin::State(prop_idx);

            let state_data_view =
                if let Some(current_view) = state_data[source_idx].get(&state_origin) {
                    current_view.create_new_read_only_view()
                } else {
                    // Use the default value for the prop and set came_from_default to true
                    let initial_data = components[component_idx]
                        .borrow()
                        .get_prop(prop_idx)
                        .unwrap()
                        .default();

                    let initial_data = InitialStateData::Single {
                        value: initial_data,
                        came_from_default: true,
                    };

                    let new_view = create_state_data_for(
                        source_idx,
                        state_origin.clone(),
                        initial_data,
                        state_data,
                    );

                    new_view.create_new_read_only_view()
                };

            DependenciesCreatedForDataQuery(vec![Dependency {
                source: DependencySource::State {
                    component_idx: source_idx,
                    origin: state_origin,
                },
                value: state_data_view,
            }])
        }

        DataQuery::Prop {
            component_idx: comp_idx,
            prop_idx,
        } => {
            // Create a dependency that references the value of prop_idx from comp_idx

            let comp_idx = comp_idx.unwrap_or(component_idx);

            let comp = components[comp_idx].borrow();

            DependenciesCreatedForDataQuery(vec![Dependency {
                source: DependencySource::Prop {
                    component_idx: comp_idx,
                    prop_idx: *prop_idx,
                },
                value: comp
                    .get_prop(*prop_idx)
                    .unwrap()
                    .create_new_read_only_view(),
            }])
        }

        DataQuery::ParentProp { prop_name } => {
            // Create a dependency that references the value of prop_name
            // from the parent of this component

            let component = components[component_idx].borrow();
            let parent_idx = component
                .get_parent()
                .expect("Component asks for a parent but there is none.");

            let parent = components[parent_idx].borrow();

            let prop_idx = parent
                .get_prop_index_from_name(prop_name)
                .unwrap_or_else(|| panic!("Invalid prop 2: {}", prop_name));

            DependenciesCreatedForDataQuery(vec![Dependency {
                source: DependencySource::Prop {
                    component_idx: parent_idx,
                    prop_idx,
                },
                value: parent
                    .get_prop(prop_idx)
                    .unwrap()
                    .create_new_read_only_view(),
            }])
        }

        DataQuery::ChildPropProfile {
            match_profiles,
            always_return_value,
        } => {
            let always_return_value = *always_return_value;

            // Create a dependency from all children
            // that match a profile from match_profiles.
            // The dependency for each child will be a view of the matching prop.

            // We address two possible extend sources
            // 1. If there is a prop extend source extending this prop that came from a direct ref,
            //    and that prop matches `match_profiles`, then the first dependency
            //    will be that prop.
            //    (If the prop extend came from an extend attribute, then the prop was already added to children.)
            // 2. If, instead, there is a component extend source, then include the children
            //    from the extend source in the list of children (starting with extend source children)

            let mut dependencies = Vec::new();

            // If extending from a prop that matches a profile and was added as a direct ref,
            // then add that prop as a dependency
            if let Some(Extending::Prop(prop_source)) =
                components[component_idx].borrow().get_extending()
            {
                if prop_source.from_direct_ref {
                    let prop_pointer = prop_source.prop_pointer;
                    let referent = components[prop_pointer.component_idx].borrow();
                    let referent_prop = referent.get_prop(prop_pointer.prop_idx).unwrap();
                    let referent_profile = referent_prop.get_matching_component_profile();

                    if match_profiles.contains(&referent_profile) {
                        dependencies.push(Dependency {
                            source: prop_pointer.into(),
                            value: referent_prop.create_new_read_only_view(),
                        });
                    }
                }
            }

            // To address a potential component extend source, we get all children,
            // including those from a component extend source.
            let children_info =
                get_children_with_parent_including_from_extend_source(components, component_idx);

            // Stores how many string children added per parent.
            // Use it to generate the index for the StatePropDataOrigin so it points to the right string child
            let mut state_data_numbering: HashMap<ComponentIdx, usize> = HashMap::new();

            for child_info in children_info.iter() {
                match child_info {
                    (UntaggedContent::Ref(child_idx), _parent_idx) => {
                        let child = components[*child_idx].borrow();

                        // Iterate through all the child's component profile props
                        // to see if one matches matches_profile.

                        child
                            .get_component_profile_prop_indices()
                            .into_iter()
                            .find_map(|prop_idx| {
                                let child_prop = child.get_prop(prop_idx).unwrap();
                                let child_profile = child_prop.get_matching_component_profile();

                                if match_profiles.contains(&child_profile) {
                                    dependencies.push(Dependency {
                                        source: DependencySource::Prop {
                                            component_idx: *child_idx,
                                            prop_idx,
                                        },
                                        value: child_prop.create_new_read_only_view(),
                                    });
                                    Some(())
                                } else {
                                    None
                                }
                            });
                    }
                    (UntaggedContent::Text(string_value), parent_idx) => {
                        // Text children are just strings, and they just match the String or LiteralString profiles
                        if match_profiles.contains(&ComponentProfile::String)
                            || match_profiles.contains(&ComponentProfile::LiteralString)
                        {
                            let parent_idx = *parent_idx;

                            let index = state_data_numbering.entry(parent_idx).or_insert(0_usize);

                            let state_origin = StatePropDataOrigin::StringChild(*index);

                            let state_data_view = if let Some(current_view) =
                                state_data[parent_idx].get(&state_origin)
                            {
                                current_view.create_new_read_only_view()
                            } else {
                                let value = PropValue::String(string_value.clone());
                                let new_view = create_state_data_for(
                                    parent_idx,
                                    state_origin.clone(),
                                    InitialStateData::Single {
                                        value,
                                        came_from_default: false,
                                    },
                                    state_data,
                                );
                                new_view.create_new_read_only_view()
                            };

                            dependencies.push(Dependency {
                                source: DependencySource::State {
                                    component_idx: parent_idx,
                                    origin: state_origin,
                                },
                                value: state_data_view,
                            });

                            *index += 1;
                        }
                    }
                }
            }

            if always_return_value && dependencies.is_empty() {
                // Found no matching children.
                // Create a state prop dependency with the default_value for the prop

                // For the state prop data origin, recurse to extend source components
                // in order to share the state data with the extend source.
                let source_idx =
                    get_same_type_component_extend_source_origin(components, component_idx);
                let state_origin = StatePropDataOrigin::ChildSubstitute(prop_idx);

                let state_data_view = if let Some(current_view) =
                    state_data[source_idx].get(&state_origin)
                {
                    current_view.create_new_read_only_view()
                } else {
                    let source_comp = components[source_idx].borrow();
                    let source_prop = source_comp.get_prop(prop_idx).unwrap();

                    // If the prop of prop_idx is in match_profiles,
                    // then create a variable of the same type with the prop's default value.
                    // Note: the type of the state prop created, below,
                    // depends on the type of the initial value.
                    let initial_data =
                        if match_profiles.contains(&source_prop.get_matching_component_profile()) {
                            source_prop.default()
                        } else {
                            // Since the prop wasn't in match_profiles,
                            // create a prop of the type from the first match_profile
                            // and use the default value associated with that type
                            match_profiles[0].default()
                        };

                    let new_view = create_state_data_for(
                        source_idx,
                        state_origin.clone(),
                        InitialStateData::Single {
                            value: initial_data,
                            came_from_default: true,
                        },
                        state_data,
                    );
                    new_view.create_new_read_only_view()
                };

                dependencies.push(Dependency {
                    source: DependencySource::State {
                        component_idx: source_idx,
                        origin: state_origin,
                    },
                    value: state_data_view,
                });
            }

            DependenciesCreatedForDataQuery(dependencies)
        }

        DataQuery::Attribute {
            attribute_name,
            match_profiles,
            always_return_value,
        } => {
            let always_return_value = *always_return_value;

            // Create a dependency from all attribute components
            // that match a profile from match_profiles.
            // The dependency for each child will be a view of the matching prop.

            let (attribute_components, parent_idx) =
                get_attribute_with_parent_falling_back_to_extend_source(
                    components,
                    component_idx,
                    attribute_name,
                )
                .unwrap_or_else(|| (Vec::new(), component_idx));

            // Stores how many string children added.
            // Use it to generate the index for the StatePropDataOrigin so it points to the right string child
            let mut state_data_index = 0;

            let mut dependencies: Vec<_> = attribute_components
                .iter()
                .filter_map(|child| {
                    match child {
                        UntaggedContent::Ref(child_idx) => {
                            let child_comp = components[*child_idx].borrow();

                            child_comp
                                .get_component_profile_prop_indices()
                                .into_iter()
                                .find_map(|prop_idx| {
                                    let child_prop = child_comp.get_prop(prop_idx).unwrap();
                                    let child_profile = child_prop.get_matching_component_profile();

                                    if match_profiles.contains(&child_profile) {
                                        Some(Dependency {
                                            source: DependencySource::Prop {
                                                component_idx: *child_idx,
                                                prop_idx,
                                            },
                                            value: child_prop.create_new_read_only_view(),
                                        })
                                    } else {
                                        None
                                    }
                                })
                        }
                        UntaggedContent::Text(string_value) => {
                            // Text children are just strings, and they just match the String or LiteralString profiles
                            if match_profiles.contains(&ComponentProfile::String)
                                || match_profiles.contains(&ComponentProfile::LiteralString)
                            {
                                let state_origin = StatePropDataOrigin::Attribute(
                                    attribute_name,
                                    state_data_index,
                                );
                                state_data_index += 1;

                                // State data is from parent_idx, so it will be shared with the extend_source
                                // if the children came from an extend_source
                                let state_data_view = if let Some(current_view) =
                                    state_data[parent_idx].get(&state_origin)
                                {
                                    current_view.create_new_read_only_view()
                                } else {
                                    let value = PropValue::String(string_value.clone());
                                    let new_view = create_state_data_for(
                                        parent_idx,
                                        state_origin.clone(),
                                        InitialStateData::Single {
                                            value,
                                            came_from_default: false,
                                        },
                                        state_data,
                                    );
                                    new_view.create_new_read_only_view()
                                };

                                Some(Dependency {
                                    source: DependencySource::State {
                                        component_idx: parent_idx,
                                        origin: state_origin,
                                    },
                                    value: state_data_view,
                                })
                            } else {
                                None
                            }
                        }
                    }
                })
                .collect();

            if always_return_value && dependencies.is_empty() {
                // Found no matching attribute children.
                // This means that the component and any component extend sources do not have any attribute children.

                // For the state data origin, recurse to extend source components
                // in order to share the state data with the extend source.
                let source_idx =
                    get_same_type_component_extend_source_origin(components, component_idx);
                let state_origin =
                    StatePropDataOrigin::AttributeSubstitute(attribute_name, prop_idx);

                let state_data_view = if let Some(current_view) =
                    state_data[source_idx].get(&state_origin)
                {
                    current_view.create_new_read_only_view()
                } else {
                    let source_comp = components[source_idx].borrow();
                    let source_prop = source_comp.get_prop(prop_idx).unwrap();

                    // If the prop of prop_idx is in match_profiles,
                    // then create a variable of the same type with the prop's default value.
                    // Note: the type of state variable created, below,
                    // depends on the type of the initial value.
                    let initial_data =
                        if match_profiles.contains(&source_prop.get_matching_component_profile()) {
                            source_prop.default()
                        } else {
                            // Since the prop wasn't in match_profiles,
                            // create a prop of the type from the first match_profile
                            // and use the default value associated with that type
                            match_profiles[0].default()
                        };

                    let new_view = create_state_data_for(
                        source_idx,
                        state_origin.clone(),
                        InitialStateData::Single {
                            value: initial_data,
                            came_from_default: true,
                        },
                        state_data,
                    );
                    new_view.create_new_read_only_view()
                };

                dependencies.push(Dependency {
                    source: DependencySource::State {
                        component_idx: source_idx,
                        origin: state_origin,
                    },
                    value: state_data_view,
                });
            }

            DependenciesCreatedForDataQuery(dependencies)
        }
    }
}
