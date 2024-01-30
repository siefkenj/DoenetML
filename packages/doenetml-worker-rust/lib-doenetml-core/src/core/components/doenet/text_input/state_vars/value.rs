use crate::components::prelude::*;

use super::TextInputState;

/// The dependencies of the value state variable of the text input component
#[add_dependency_data]
#[derive(Debug, Default, StateVariableDependencies, StateVariableDataQueries)]
struct RequiredData {
    essential: StateVarView<String>,
    immediate_value: StateVarView<String>,
    sync_immediate_value: StateVarView<bool>,
    bind_value_to: StateVarView<String>,
    prefill: StateVarView<String>,
}

/// The interface for the value state variable of a text input
#[derive(Debug, Default)]
pub struct ValueStateVar {
    /// The graph queries that indicate how the dependencies of this state variable will be created.
    data_queries: RequiredDataDataQueries,

    /// The values of the dependencies created from the graph queries
    query_results: RequiredData,
}

impl ValueStateVar {
    pub fn new() -> Self {
        ValueStateVar {
            ..Default::default()
        }
    }
}

impl From<ValueStateVar> for StateVar<String> {
    fn from(interface: ValueStateVar) -> Self {
        StateVar::new(Box::new(interface), Default::default())
    }
}

impl StateVarUpdater<String> for ValueStateVar {
    fn return_data_queries(
        &mut self,
        _extending: Option<ExtendSource>,
        _state_var_idx: StateVarIdx,
    ) -> Vec<DataQuery> {
        self.data_queries = RequiredDataDataQueries {
            essential: Some(DataQuery::Essential),
            immediate_value: Some(TextInputState::get_immediate_value_data_queries()),
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
        let value = if *self.query_results.sync_immediate_value.get() {
            self.query_results.immediate_value.get().clone()
        } else if self.query_results.bind_value_to.came_from_default() {
            if self.query_results.essential.came_from_default() {
                self.query_results.prefill.get().clone()
            } else {
                self.query_results.essential.get().clone()
            }
        } else {
            self.query_results.bind_value_to.get().clone()
        };

        StateVarCalcResult::Calculated(value)
    }

    fn invert(
        &mut self,
        state_var: &StateVarView<String>,
        _is_direct_change_from_renderer: bool,
    ) -> Result<Vec<DependencyValueUpdateRequest>, RequestDependencyUpdateError> {
        let requested_value = state_var.get_requested_value();

        let query_results = &mut self.query_results;

        let bind_value_to_came_from_default = query_results.bind_value_to.came_from_default();

        if bind_value_to_came_from_default {
            query_results
                .essential
                .queue_update(requested_value.clone());
            query_results
                .immediate_value
                .queue_update(requested_value.clone());
            query_results.sync_immediate_value.queue_update(true);
        } else {
            query_results
                .bind_value_to
                .queue_update(requested_value.clone());
            query_results.sync_immediate_value.queue_update(true);
        }

        Ok(query_results.return_queued_updates())
    }
}
