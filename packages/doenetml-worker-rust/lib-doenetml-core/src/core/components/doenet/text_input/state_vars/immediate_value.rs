use crate::components::prelude::*;

use super::TextInputState;

/// The dependencies of the immediate_value state variable of the text input component
#[add_dependency_data]
#[derive(Debug, Default, StateVariableDependencies, StateVariableDataQueries)]
struct RequiredData {
    essential: StateVarView<String>,
    sync_immediate_value: StateVarView<bool>,
    bind_value_to: StateVarView<String>,
    prefill: StateVarView<String>,
}

/// The interface for the immediate_value state variable of a text input
#[derive(Debug, Default)]
pub struct ImmediateValueStateVar {
    /// The graph queries that indicate how the dependencies of this state variable will be created.
    data_queries: RequiredDataDataQueries,

    /// The values of the dependencies created from the graph queries
    query_results: RequiredData,
}

impl ImmediateValueStateVar {
    pub fn new() -> Self {
        ImmediateValueStateVar {
            ..Default::default()
        }
    }
}

impl From<ImmediateValueStateVar> for StateVar<String> {
    fn from(interface: ImmediateValueStateVar) -> Self {
        StateVar::new(Box::new(interface), Default::default())
    }
}

impl StateVarUpdater<String> for ImmediateValueStateVar {
    fn return_data_queries(
        &mut self,
        _extending: Option<ExtendSource>,
        _state_var_idx: StateVarIdx,
    ) -> Vec<DataQuery> {
        self.data_queries = RequiredDataDataQueries {
            essential: Some(DataQuery::Essential),
            sync_immediate_value: Some(TextInputState::get_sync_immediate_value_data_queries()),
            bind_value_to: Some(TextInputState::get_bind_value_to_data_queries()),
            prefill: Some(TextInputState::get_prefill_data_queries()),
        };

        (&self.data_queries).into()
    }

    fn save_query_results(&mut self, dependencies: &Vec<DependenciesCreatedForDataQuery>) {
        self.query_results = dependencies.try_into().unwrap();
    }

    fn calculate(&self) -> StateVarCalcResult<String> {
        let immediate_value = if !self.query_results.bind_value_to.came_from_default()
            && *self.query_results.sync_immediate_value.get()
        {
            self.query_results.bind_value_to.get().clone()
        } else if self.query_results.essential.came_from_default() {
            self.query_results.prefill.get().clone()
        } else {
            self.query_results.essential.get().clone()
        };
        StateVarCalcResult::Calculated(immediate_value)
    }

    fn invert(
        &mut self,
        state_var: &StateVarView<String>,
        is_direct_change_from_renderer: bool,
    ) -> Result<Vec<DependencyValueUpdateRequest>, RequestDependencyUpdateError> {
        let requested_value = state_var.get_requested_value();

        let bind_value_to_came_from_default = self.query_results.bind_value_to.came_from_default();

        self.query_results
            .essential
            .queue_update(requested_value.clone());

        if !is_direct_change_from_renderer && !bind_value_to_came_from_default {
            self.query_results
                .bind_value_to
                .queue_update(requested_value.clone());
        }

        Ok(self.query_results.return_queued_updates())
    }
}
