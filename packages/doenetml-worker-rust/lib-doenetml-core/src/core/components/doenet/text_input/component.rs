use serde::{Deserialize, Serialize};
use strum::VariantNames;
use strum_macros::EnumVariantNames;

use super::TextInputStateVariables;
use crate::components::{actions::ActionBody, prelude::*, ActionsEnum};

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "web", derive(tsify::Tsify))]
#[cfg_attr(feature = "web", tsify(from_wasm_abi))]
#[serde(expecting = "`text` must be a string")]
pub struct TextInputActionArgs {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, EnumVariantNames)]
#[serde(tag = "actionName", rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[cfg_attr(feature = "web", derive(tsify::Tsify))]
#[cfg_attr(feature = "web", tsify(from_wasm_abi))]
pub enum TextInputAction {
    UpdateImmediateValue(ActionBody<TextInputActionArgs>),
    UpdateValue,
}

/// Definition of the `<textInput>` DoenetML component
#[derive(Debug, Default, ComponentNode, ComponentStateVariables)]
pub struct TextInput {
    /// The common component data needed to derive the `ComponentNode` trait
    pub common: ComponentCommonData,

    /// The state variables that underlie the `<textInput>` component.
    pub state: TextInputStateVariables,

    /// An empty vector that will be returned with `get_rendered_children`
    /// indicating this component has no children that are rendered.
    ///
    /// (Created because `get_rendered_children` must return a reference to a vector,)
    pub no_rendered_children: Vec<ComponentPointerTextOrMacro>,
}

impl RenderedComponentNode for TextInput {
    fn get_rendered_children(&self) -> &Vec<ComponentPointerTextOrMacro> {
        &self.no_rendered_children
    }

    fn get_attribute_names(&self) -> Vec<AttributeName> {
        vec!["bindValueTo", "hide", "disabled", "prefill"]
    }

    fn get_action_names(&self) -> Vec<String> {
        TextInputAction::VARIANTS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn on_action(
        &self,
        action: ActionsEnum,
        resolve_and_retrieve_state_var: &mut dyn FnMut(StateVarIdx) -> StateVarValueEnum,
    ) -> Result<Vec<(StateVarIdx, StateVarValueEnum)>, String> {
        // The type of `action` should have already been verified, so an
        // error here is a programming logic error, not an API error.
        let action: TextInputAction = action.try_into()?;

        match action {
            TextInputAction::UpdateImmediateValue(ActionBody { args }) => Ok(vec![
                (
                    TextInputStateVariables::get_immediate_value_state_variable_index(),
                    StateVarValueEnum::String(args.text),
                ),
                (
                    TextInputStateVariables::get_sync_immediate_value_state_variable_index(),
                    StateVarValueEnum::Boolean(false),
                ),
            ]),

            TextInputAction::UpdateValue => {
                let new_val = resolve_and_retrieve_state_var(
                    TextInputStateVariables::get_immediate_value_state_variable_index(),
                );

                Ok(vec![(
                    TextInputStateVariables::get_value_state_variable_index(),
                    new_val,
                )])
            }
        }
    }
}
