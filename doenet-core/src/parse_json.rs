use serde::{Serialize, Deserialize};

use crate::{Action, DoenetMLError};
use crate::component::{Attribute, AttributeDefinition, CopySource, generate_component_definitions};
use crate::prelude::*;

use crate::ComponentDefinition;
use crate::ComponentChild;
use crate::ComponentNode;

use std::collections::HashMap;

use crate::state_variables::*;

// Structures for create_components_tree_from_json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComponentTree {
    component_type: String,
    props: Props,
    children: Vec<ComponentOrString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Props {
    name: Option<String>,
    copy_source: Option<String>, //this will become copy_source
    prop: Option<String>,
    #[serde(flatten)]
    attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum AttributeValue {
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ComponentOrString {
    Component(ComponentTree),
    String(String),
}


// Structures for parse_action_from_json
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionStructure {
    component_name: String,
    action_name: String,
    args: HashMap<String, ArgValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ArgValue {
    Bool(bool),
    Number(serde_json::Number),
    String(String),
}


fn get_key_value_ignore_case<'a, K, V>(map: &'a HashMap<K, V>, key: &str) -> Option<(&'a K, &'a V)>
where
    K: ToString + std::cmp::Eq + std::hash::Hash,
{
    let lowercase_to_normalized: HashMap<String, &K> = map
        .keys()
        .into_iter()
        .map(|k| (k.to_string().to_lowercase(), k))
        .collect();

    lowercase_to_normalized
        .get(&key.to_string().to_lowercase())
        .and_then(|k| map.get_key_value(k))
}

pub fn parse_action_from_json(action: &str) -> Result<Action, String> {

    // log_debug!("Parsing string for action: {}", action);

    let action_structure: ActionStructure = serde_json::from_str(action).map_err(|e| e.to_string())?;

    let component_name = action_structure.component_name.clone();
    let action_name = action_structure.action_name.clone();
    let args = action_structure.args.iter()
        .map(|(k, v)| 
             match v {
                 ArgValue::Bool(v) => (k.clone(), StateVarValue::Boolean(*v)),
                 ArgValue::String(v) => (k.clone(), StateVarValue::String(v.clone())),
                 ArgValue::Number(v) => (k.clone(), if v.is_i64() {
                     StateVarValue::Integer(v.as_i64().unwrap())
                 } else {
                     StateVarValue::Number(v.as_f64().unwrap())
                 }),
             })
        .collect();

    Ok(Action { component_name, action_name, args})
}


pub fn create_components_tree_from_json(program: &str)
    -> (HashMap<String, ComponentNode>, String, Vec<DoenetMLError>) {

    // log_debug!("Parsing string for component tree: {}", program);

    let component_definitions = generate_component_definitions();

    let all_state_var_names: HashMap<String, StateVarName> = component_definitions
        .iter()
        .flat_map(|(_, &def)|
            def.state_var_definitions
            .iter()
            .map(|(name, _)| (name.to_string(), *name))
            .collect::<HashMap<String, StateVarName>>())
        .collect();

    let component_tree: Vec<ComponentOrString> = serde_json::from_str(program)
        // This fails if there is a problem with the parser, not the input doenetML,
        // so we don't throw a doenetML error here
        .expect("Error extracting json");

    let first_component = component_tree.iter()
        .find_map(|v| match v {
            ComponentOrString::Component(tree) => Some(tree),
            _ => None,
        });
    
    let tree_wrapped_in_document = |orig_comp_tree | ComponentTree {
        component_type: String::from("document"),
        props: Props { name: None, copy_source: None, prop: None, attributes: HashMap::new() },
        children: orig_comp_tree,
    };

    let component_tree = if let Some(first_comp) = first_component {
        if first_comp.component_type == "document" {
            first_comp.clone()

        } else {
            tree_wrapped_in_document(component_tree)
        }
    } else {
        tree_wrapped_in_document(component_tree)
    };

    log_json!(format!("Parsed JSON into tree"), component_tree);

    let mut component_type_counter: HashMap<String, u32> = HashMap::new();
    let mut component_nodes: HashMap<String, ComponentNode> = HashMap::new();

    let mut doenet_ml_errors: Vec<DoenetMLError> = vec![];

    let root_component_name = add_component_from_json(
        &mut component_nodes,
        &component_tree,
        None,
        &mut component_type_counter,
        &component_definitions,
        &all_state_var_names,
        &mut doenet_ml_errors,
    );

    let root_component_name = root_component_name.unwrap();

    (component_nodes, root_component_name, doenet_ml_errors)
}


/// Recursive function
/// The return is the name of the child, if it exists
/// (it might not because of invalid doenet ml)
fn add_component_from_json(
    component_nodes: &mut HashMap<String, ComponentNode>,
    component_tree: &ComponentTree,
    parent_name: Option<String>,
    component_type_counter: &mut HashMap<String, u32>,
    component_definitions: &HashMap<ComponentType, &'static ComponentDefinition>,
    all_state_var_names: &HashMap<String, StateVarName>,
    doenet_ml_errors: &mut Vec<DoenetMLError>,

) -> Option<String> {

    let component_type: &str = &component_tree.component_type;

    let (&component_type, &component_definition) = {
        if let Some(comp_type_def) = get_key_value_ignore_case(component_definitions, component_type) {
            log!("match {} {}", component_type, comp_type_def.0);
            comp_type_def
        } else {
            doenet_ml_errors.push(
                DoenetMLError::InvalidComponentType { comp_type: component_type.to_string() }
            );
            return None;
        }
    };


    let count = *component_type_counter.get(component_type).unwrap_or(&0);
    component_type_counter.insert(component_type.to_string(), count + 1);

    let component_name = match &component_tree.props.name {
        Some(name) => name.clone(),
        None => format!("/_{}{}", component_type, count + 1),
    };


    let copy_source: Option<CopySource> = {
        if let Some(ref source_name) = component_tree.props.copy_source {
            if let Some(ref source_state_var) = component_tree.props.prop {

                if let Some(state_var_name) = all_state_var_names.get(source_state_var) {
                    // TODO: parse non-basic props
                    Some(CopySource::StateVar(source_name.clone(), StateVarReference::Basic(state_var_name)))

                } else {
                    doenet_ml_errors.push(DoenetMLError::StateVarDoesNotExist {
                        comp_name: component_name.clone(),
                        sv_name: source_state_var.to_string() 
                    });

                    None
                }

            } else {
                Some(CopySource::Component(source_name.clone()))
            }
        } else {
            None
        }
    };

    let mut attributes: HashMap<AttributeName, Attribute> = HashMap::new();

    let attribute_definitions = component_definition.attribute_definitions;

    for (attr_name, attr_value) in component_tree.props.attributes.iter() {

        let (attribute_name, attribute_def) =
        if let Some((name, def)) = get_key_value_ignore_case(attribute_definitions, &attr_name.to_lowercase()) {
            // This component has an attribute that matches the input name
            (name, def)
        } else {
            doenet_ml_errors.push(DoenetMLError::AttributeDoesNotExist {
                comp_name: component_name.clone(), attr_name: attr_name.to_string()
            });
            continue;
        };

        match attribute_def {
            AttributeDefinition::Component(attr_comp_type) => {

                let attr_component_definition = component_definitions.get(attr_comp_type).expect(
                    &format!("The definition of the {} component defined an attribute of type {}, but that type does not exist", component_type, attr_comp_type)
                );

                //String child
                let string_child = ComponentChild::String(match attr_value {
                    AttributeValue::Bool(v) => v.to_string(),
                    AttributeValue::String(v) => v.to_string(),
                });

                // Make sure this is unique
                let attr_comp_name = format!("__attr:{}:{}", component_name, attribute_name);


                let attribute_component_node = ComponentNode {
                    name: attr_comp_name.clone(),
                    parent: Some(component_name.clone()),
                    children: vec![string_child],
            
                    component_type: attr_comp_type,
                    attributes: HashMap::new(),
            
                    copy_source: None,

                    definition: attr_component_definition.clone(),
                };


                attributes.insert(attribute_name, Attribute::Component(attr_comp_name.clone()));
                component_nodes.insert(attr_comp_name, attribute_component_node);

            },

            AttributeDefinition::Primitive(attr_primitive_type) => {

                match attr_primitive_type {
                    StateVarValueType::Boolean => {

                        match attr_value {
                            AttributeValue::Bool(bool_value) => {

                                attributes.insert(attribute_name,                                Attribute::Primitive(StateVarValue::Boolean(*bool_value)));
                            }

                            _ => {
                                panic!("Attribute {} has the wrong type", attribute_name)

                            }
                        }

                    },

                    _ => {
                        log!("Primitive non-bool attribute definition does nothing right now");
                    }
                }

            }
        }

    }


    // Recurse the children

    let mut children: Vec<ComponentChild> = Vec::new();

    for child in &component_tree.children {

        match child {
            ComponentOrString::String(child_string) => {
                children.push(ComponentChild::String(child_string.to_string()));
            },

            ComponentOrString::Component(child_tree) => {
                let child_name_if_not_error = add_component_from_json(
                    component_nodes,
                    &child_tree,
                    Some(component_name.clone()),
                    component_type_counter,
                    component_definitions,
                    all_state_var_names,
                    doenet_ml_errors
                );

                if let Some(child_name) = child_name_if_not_error {
                    children.push(ComponentChild::Component(child_name));
                }

            },
        }
    }


    let component_node = ComponentNode {
        name: component_name.clone(),
        parent: parent_name,
        children,

        component_type,
        attributes,

        copy_source,

        definition: component_definition.clone(),
    };

    component_nodes.insert(component_name.clone(), component_node);

    return Some(component_name);
}