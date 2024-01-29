use crate::{components::prelude::*, dependency::DependencySource, ExtendSource};

use super::util::create_graph_query_if_match_extend_source;

/// A string state variable interface that concatenates all string dependencies.
///
/// If the component has an extend source so that this variable is shadowing another variable,
/// then prepend the shadowed state variable to the list of dependencies.
///
/// If the state variable has a single dependency that is an essential state variable,
/// then propagate the `came_from_default` attribute of the essential state variable.
#[derive(Debug, Default)]
pub struct StringStateVar {
    /// The base graph query that indicates how the dependencies of this state variable will be created.
    base_graph_query: GraphQuery,

    /// The base graph query, potentially augmented by a graph query
    /// for shadowing another variable
    graph_queries: GeneralStringStateVarGraphQueries,

    /// The values of the dependencies created from the graph queries
    query_results: GeneralStringStateVarDependencies,

    /// If true, there is just a single dependency that is an essential state variable.
    /// In this case, we'll propagate the `came_from_default` attribute of the essential state variable.
    from_single_essential: bool,
}

/// The values of the dependencies that were created from the graph queries
#[add_dependency_data]
#[derive(Debug, Default, StateVariableDependencies)]
struct GeneralStringStateVarDependencies {
    /// A vector of the string values of the dependencies
    #[consume_remaining_instructions]
    strings: Vec<StateVarReadOnlyView<String>>,
}

/// The graph queries that indicate how the dependencies of this state variable will be created.
/// They consist of the base graph query specified, potentially augmented by a graph query
/// for shadowing another variable
#[derive(Debug, Default, StateVariableGraphQueries)]
struct GeneralStringStateVarGraphQueries {
    /// If present, `extending` contains an instruction requesting the value of another text variable.
    /// It was created from the extend source for this component.
    extending: Option<GraphQuery>,

    /// The base graph query specified for this variable.
    ///
    /// (It is always present. It is an option only to satisfy the API for
    /// the `StateVariableGraphQueries` derive macro.)
    other: Option<GraphQuery>,
}

impl StringStateVar {
    /// Creates a state var that queries its value from the given graph query.
    pub fn new(base_graph_query: GraphQuery) -> Self {
        StringStateVar {
            base_graph_query,
            ..Default::default()
        }
    }

    /// Creates a state var that queries its value from children matching the `Text` profile.
    pub fn new_from_children() -> Self {
        StringStateVar {
            base_graph_query: GraphQuery::Child {
                match_profiles: vec![ComponentProfile::Text],
                exclude_if_prefer_profiles: vec![],
            },
            ..Default::default()
        }
    }

    /// Creates a state var that queries its value from attributes matching the `Text` profile.
    pub fn new_from_attribute(attr_name: AttributeName) -> Self {
        StringStateVar {
            base_graph_query: GraphQuery::AttributeChild {
                attribute_name: attr_name,
                match_profiles: vec![ComponentProfile::Text],
            },
            ..Default::default()
        }
    }
}

impl From<StringStateVar> for StateVar<String> {
    fn from(interface: StringStateVar) -> Self {
        StateVar::new(Box::new(interface), Default::default())
    }
}

impl StateVarInterface<String> for StringStateVar {
    fn return_graph_queries(
        &mut self,
        extending: Option<ExtendSource>,
        state_var_idx: StateVarIdx,
    ) -> Vec<GraphQuery> {
        self.graph_queries = GeneralStringStateVarGraphQueries {
            extending: create_graph_query_if_match_extend_source(extending, state_var_idx),
            other: Some(self.base_graph_query.clone()),
        };

        (&self.graph_queries).into()
    }

    fn save_dependencies(&mut self, dependencies: &Vec<DependenciesCreatedForInstruction>) {
        self.query_results = dependencies.try_into().unwrap();

        if self.query_results.strings.len() == 1 {
            match dependencies[0][0].source {
                DependencySource::Essential { .. } => {
                    self.from_single_essential = true;
                }
                _ => {}
            }
        }
    }

    fn calculate_state_var_from_dependencies(&self) -> StateVarCalcResult<String> {
        if self.from_single_essential {
            if self.query_results.strings[0].came_from_default() {
                // if we are basing it on a single essential variable that came from default,
                // then we propagate came_from_default as well as the value.
                return StateVarCalcResult::FromDefault(
                    self.query_results.strings[0].get().clone(),
                );
            } else {
                return StateVarCalcResult::Calculated(self.query_results.strings[0].get().clone());
            }
        } else {
            // TODO: can we implement this without cloning the inner value?
            let value: String = self
                .query_results
                .strings
                .iter()
                .map(|v| v.get().clone())
                .collect();

            StateVarCalcResult::Calculated(value)
        }
    }

    fn request_dependency_updates(
        &mut self,
        state_var: &StateVarReadOnlyView<String>,
        _is_direct_change_from_renderer: bool,
    ) -> Result<Vec<DependencyValueUpdateRequest>, RequestDependencyUpdateError> {
        if self.query_results.strings.len() != 1 {
            // TODO: implement for no dependencies where saves to essential value?
            Err(RequestDependencyUpdateError::CouldNotUpdate)
        } else {
            let requested_value = state_var.get_requested_value();

            self.query_results.strings[0].queue_update(requested_value.clone());

            Ok(self.query_results.return_queued_updates())
        }
    }
}
