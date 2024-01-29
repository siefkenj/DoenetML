use crate::components::prelude::*;

/// The dependencies of the sync_immediate_value state variable of the text input component
#[add_dependency_data]
#[derive(Debug, Default, StateVariableDependencies, StateVariableGraphQueries)]
struct RequiredData {
    essential: StateVarReadOnlyView<bool>,
}

/// The interface for the sync_immediate_value state variable of a text input
#[derive(Debug, Default)]
pub struct SyncImmediateValueStateVar {
    /// The graph queries that indicate how the dependencies of this state variable will be created.
    graph_queries: RequiredDataGraphQueries,

    /// The values of the dependencies created from the graph queries
    query_results: RequiredData,
}

impl SyncImmediateValueStateVar {
    pub fn new() -> Self {
        SyncImmediateValueStateVar {
            ..Default::default()
        }
    }
}

impl From<SyncImmediateValueStateVar> for StateVar<bool> {
    fn from(interface: SyncImmediateValueStateVar) -> Self {
        StateVar::new(Box::new(interface), true)
    }
}

impl StateVarUpdaters<bool> for SyncImmediateValueStateVar {
    fn return_graph_queries(
        &mut self,
        _extending: Option<ExtendSource>,
        _state_var_idx: StateVarIdx,
    ) -> Vec<GraphQuery> {
        self.graph_queries = RequiredDataGraphQueries {
            essential: Some(GraphQuery::Essential),
        };

        (&self.graph_queries).into()
    }

    fn save_dependencies(&mut self, dependencies: &Vec<DependenciesCreatedForInstruction>) {
        self.query_results = dependencies.try_into().unwrap();
    }

    fn calculate_state_var_from_dependencies(&self) -> StateVarCalcResult<bool> {
        StateVarCalcResult::Calculated(*self.query_results.essential.get())
    }

    fn request_dependency_updates(
        &mut self,
        state_var: &StateVarReadOnlyView<bool>,
        _is_direct_change_from_renderer: bool,
    ) -> Result<Vec<DependencyValueUpdateRequest>, RequestDependencyUpdateError> {
        let requested_value = state_var.get_requested_value();

        self.query_results.essential.queue_update(*requested_value);

        Ok(self.query_results.return_queued_updates())
    }
}
