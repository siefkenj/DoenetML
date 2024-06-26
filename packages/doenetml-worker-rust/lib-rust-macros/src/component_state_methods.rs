use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{self, parse::Parser, FieldsNamed};

use crate::util::{find_type_from_prop_with_generics, has_attribute};

pub fn component_state_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let structure_identity = &ast.ident;
    let data = &ast.data;

    let output = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // eprintln!("named: {:#?}", named);

                let field_identities = named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().clone())
                    .collect::<Vec<_>>();

                // Determine if struct represents a component or represents props.
                // If a component, then we implement the trait by calling each function on the state.
                // If props, then determine the trait functions from the fields.
                //
                // TODO: better condition to determine if is component struct.
                // For now, we just check if there are fields named "common" and "state".
                // Could check the type of common to make sure it is CommonData
                // or check if each field is a prop.
                let is_component_struct = field_identities.iter().any(|ident| *ident == "common")
                    && field_identities.iter().any(|ident| *ident == "state");

                if is_component_struct {
                    quote! {
                        impl ComponentState for #structure_identity {
                            fn get_num_props(&self) -> PropIdx {
                                self.state.get_num_props()
                            }

                            fn get_prop(&self, prop_idx: PropIdx) -> Option<PropEnumRef> {
                                self.state.get_prop(prop_idx)
                            }

                            fn get_prop_mut(&mut self, prop_idx: PropIdx) -> Option<PropEnumRefMut> {
                                self.state.get_prop_mut(prop_idx)
                            }

                            fn get_prop_index_from_name(&self, name: &str) -> Option<PropIdx> {
                                self.state.get_prop_index_from_name(name)
                            }

                            fn get_component_profile_prop_indices(&self) -> Vec<PropIdx> {
                                self.state.get_component_profile_prop_indices()
                            }

                            fn get_public_prop_index_from_name_case_insensitive(
                                &self,
                                name: &str,
                            ) -> Option<PropIdx> {
                                self.state.get_public_prop_index_from_name_case_insensitive(name)
                            }

                            fn get_default_prop(&self) -> Option<PropIdx> {
                                self.state.get_default_prop()
                            }

                            fn get_for_renderer_prop_indices(&self) -> Vec<PropIdx> {
                                self.state.get_for_renderer_prop_indices()
                            }

                            fn check_if_prop_is_for_renderer(&self, prop_idx: PropIdx) -> bool {
                                self.state.check_if_prop_is_for_renderer(prop_idx)
                            }

                            /// Return object will the values of all the rendered props
                            fn return_rendered_state(&mut self) -> Option<RenderedState> {
                                self.state.return_rendered_state()
                            }

                            fn return_rendered_state_update(&mut self) -> Option<RenderedState> {
                                self.state.return_rendered_state_update()
                            }
                        }
                    }
                } else {
                    let field_types = named
                        .iter()
                        .map(|f| find_type_from_prop_with_generics(&f.ty).unwrap())
                        .collect::<Vec<_>>();

                    let num_prop = field_identities.len();

                    let mut get_prop_arms = Vec::new();
                    let mut get_prop_mut_arms = Vec::new();
                    let mut get_prop_index_from_name_arms = Vec::new();
                    let mut get_public_prop_index_from_name_case_insensitive_arms = Vec::new();
                    let mut get_component_profile_prop_indices_items = Vec::new();
                    let mut get_for_renderer_prop_indices_items = Vec::new();
                    let mut check_if_prop_is_for_renderer_arms = Vec::new();
                    let mut return_rendered_state_items = Vec::new();
                    let mut return_rendered_state_update_statements = Vec::new();
                    let mut rendered_props_struct_statements = Vec::new();

                    let mut get_prop_index_functions = Vec::new();
                    let mut get_value_data_query_functions = Vec::new();
                    let mut update_from_action_functions = Vec::new();

                    let mut default_prop = None;

                    let renderer_props_name = format!("Rendered{}", structure_identity);
                    let rendered_props_identity =
                        Ident::new(&renderer_props_name, Span::call_site());

                    for (prop_idx, field_identity) in field_identities.iter().enumerate() {
                        get_prop_arms.push(quote! {
                            #prop_idx => Some((&self.#field_identity).into()),
                        });
                        get_prop_mut_arms.push(quote! {
                            #prop_idx => Some((&mut self.#field_identity).into()),
                        });

                        let field_camel_case = field_identity.to_string().to_case(Case::Camel);
                        get_prop_index_from_name_arms.push(quote! {
                            #field_camel_case => Some(#prop_idx),
                        });

                        if has_attribute(&named[prop_idx].attrs, "is_public") {
                            get_public_prop_index_from_name_case_insensitive_arms.push(quote! {
                                x if x.eq_ignore_ascii_case(#field_camel_case) => Some(#prop_idx),
                            });
                        }

                        if has_attribute(&named[prop_idx].attrs, "component_profile_prop") {
                            get_component_profile_prop_indices_items.push(quote! {
                                #prop_idx,
                            });
                        }

                        if has_attribute(&named[prop_idx].attrs, "default_prop") {
                            if default_prop.is_some() {
                                panic!("Cannot define default prop on more than one prop");
                            }
                            default_prop = Some(prop_idx);
                        }

                        if has_attribute(&named[prop_idx].attrs, "for_renderer") {
                            get_for_renderer_prop_indices_items.push(quote! {
                                #prop_idx,
                            });

                            check_if_prop_is_for_renderer_arms.push(quote! {
                                #prop_idx => true,
                            });

                            return_rendered_state_items.push(quote! {
                                #field_identity: Some(self.#field_identity.get_value_mark_viewed().clone()),
                            });

                            return_rendered_state_update_statements.push(quote! {
                                if self.#field_identity.changed_since_last_viewed() {
                                    updated_variables.#field_identity =
                                        Some(self.#field_identity.get_value_mark_viewed().clone());
                                }
                            });

                            let prop_type =
                                find_type_from_prop_with_generics(&named[prop_idx].ty).unwrap();

                            rendered_props_struct_statements.push(quote! {
                                #[serde(skip_serializing_if = "Option::is_none")]
                                pub #field_identity: Option<#prop_type>,
                            })
                        }

                        let get_index_function_name = format!("get_{}_prop_index", field_identity);
                        let get_index_function_identity =
                            Ident::new(&get_index_function_name, Span::call_site());

                        let local_get_index_function_name =
                            format!("local_get_{}_prop_index", field_identity);
                        let local_get_index_function_identity =
                            Ident::new(&local_get_index_function_name, Span::call_site());

                        get_prop_index_functions.push(quote! {
                            /// Get a prop index
                            /// of the specified prop
                            pub const fn #get_index_function_identity() -> PropIdx {
                                #prop_idx
                            }
                            pub const fn #local_get_index_function_identity(&self) -> PropIdx {
                                #prop_idx
                            }
                        });

                        let get_query_function_name = format!("get_{}_data_query", field_identity);
                        let get_query_function_identity =
                            Ident::new(&get_query_function_name, Span::call_site());

                        get_value_data_query_functions.push(quote! {
                            /// Get a `DataQuery` that requests the value
                            /// of the specified prop
                            pub const fn #get_query_function_identity() -> DataQuery {
                                DataQuery::Prop {
                                    component_idx: None,
                                    prop_idx: #prop_idx,
                                }
                            }
                        });

                        let update_from_action_function_name =
                            format!("update_{}_from_action", field_identity);
                        let update_from_action_function_identity =
                            Ident::new(&update_from_action_function_name, Span::call_site());
                        let val_type = &field_types[prop_idx];

                        update_from_action_functions.push(quote! {
                            pub fn #update_from_action_function_identity(val: #val_type) -> UpdateFromAction {
                                UpdateFromAction(#prop_idx, val.clone().try_into().unwrap())
                            }
                        });
                    }

                    let default_prop_statement = match default_prop {
                        Some(prop_idx) => quote!(Some(#prop_idx)),
                        None => quote!(None),
                    };

                    quote! {
                        impl ComponentState for #structure_identity {

                            fn get_num_props(&self) -> PropIdx {
                                #num_prop
                            }

                            fn get_prop(&self, prop_idx: PropIdx) -> Option<PropEnumRef> {
                                match prop_idx {
                                    #(#get_prop_arms)*
                                    _ => None,
                                }
                            }


                            fn get_prop_mut(&mut self, prop_idx: PropIdx) -> Option<PropEnumRefMut> {
                                match prop_idx {
                                    #(#get_prop_mut_arms)*
                                    _ => None,
                                }
                            }

                            fn get_prop_index_from_name(&self, name: &str) -> Option<PropIdx> {
                                match name {
                                    #(#get_prop_index_from_name_arms)*
                                    _ => None,
                                }
                            }

                            fn get_public_prop_index_from_name_case_insensitive(
                                &self,
                                name: &str,
                            ) -> Option<PropIdx> {
                                match name {
                                    #(#get_public_prop_index_from_name_case_insensitive_arms)*
                                    _ => None,
                                }
                            }

                            fn get_component_profile_prop_indices(&self) -> Vec<PropIdx> {
                                vec![
                                    #(#get_component_profile_prop_indices_items)*
                                ]
                            }

                            fn get_default_prop(&self) -> Option<PropIdx> {
                                #default_prop_statement
                            }


                            fn get_for_renderer_prop_indices(&self) -> Vec<PropIdx> {
                                vec![
                                    #(#get_for_renderer_prop_indices_items)*
                                ]
                            }

                            fn check_if_prop_is_for_renderer(&self, prop_idx: PropIdx) -> bool {
                                match prop_idx {
                                    #(#check_if_prop_is_for_renderer_arms)*
                                    _ => false,
                                }
                            }

                            fn return_rendered_state(&mut self) -> Option<RenderedState> {
                                Some(RenderedState::#structure_identity(#rendered_props_identity {
                                    #(#return_rendered_state_items)*
                                }))
                            }

                            fn return_rendered_state_update(&mut self) -> Option<RenderedState> {
                                let mut updated_variables = #rendered_props_identity::default();

                                #(#return_rendered_state_update_statements)*

                                Some(RenderedState::#structure_identity(updated_variables))
                            }

                        }

                        /// Structure containing the values of a component's props
                        /// that were designated `for_renderer`.
                        ///
                        /// Each field is an Option so that partial data can be sent
                        /// when the values of just some variables were updated.
                        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
                        #[serde(rename_all = "camelCase")]
                        pub struct #rendered_props_identity {
                            #(#rendered_props_struct_statements)*
                        }


                        impl #structure_identity {

                            #(#get_prop_index_functions)*

                            #(#get_value_data_query_functions)*

                            #(#update_from_action_functions)*
                        }

                    }
                }
            }
            _ => panic!("only named fields supported"),
        },
        _ => panic!("only structs supported"),
    };
    output.into()
}

pub fn prop_dependencies_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let structure_identity = &ast.ident;
    let data = &ast.data;
    let generics = &ast.generics;

    let output = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // eprintln!("named: {:#?}", named);

                let field_identities = named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().clone())
                    .collect::<Vec<_>>();

                let structure_name = structure_identity.to_string();
                let sn_len = structure_name.len();

                let data_name = if structure_name.ends_with("ies") {
                    format!("{}yData", &structure_name[..(sn_len - 3)])
                } else {
                    format!("{}Data", structure_name)
                };
                let data_identity = Ident::new(&data_name, Span::call_site());

                let mut data_struct_statements = Vec::new();
                let mut initialize_data_struct_statements = Vec::new();
                let mut return_update_requests_statements = Vec::new();
                let mut mark_data_viewed_statements = Vec::new();

                for (data_query_idx, field_identity) in field_identities.iter().enumerate() {
                    if field_identity.to_string().starts_with('_') {
                        continue;
                    }

                    data_struct_statements.push(quote! {
                        #field_identity: Vec<(usize,usize)>,
                    });

                    return_update_requests_statements.push(quote! {

                        let mapping_data = &self._data_query_mapping_data.#field_identity;

                        requests.extend(self.#field_identity.indices_with_queued_updates().into_iter().map(|idx| {
                            let data = &mapping_data[idx];
                            DependencyValueUpdateRequest {
                                data_query_idx: data.0,
                                dependency_idx: data.1,
                            }
                        }));
                        self.#field_identity.clear_queued_updates();
                    });

                    initialize_data_struct_statements.push(quote! {
                        // #data_query_idx is the index of this query in the list of fields
                        // in #structure_identity.
                        for (dep_idx, dep) in dependencies[#data_query_idx].iter().enumerate() {
                            mapping_data.#field_identity.push((#data_query_idx, dep_idx));
                        }
                        data_struct.#field_identity = (&dependencies[#data_query_idx]).try_to_state().unwrap();

                    });

                    mark_data_viewed_statements.push(quote! {
                        self.#field_identity.mark_data_viewed();
                    });
                }

                // if we have a generic,
                // we also need to restrict to those with a PropView that we can try_into a PropViewEnum
                // with an error that implements Debug
                let where_clause = generics.where_clause.as_ref().map(|wc| {
                    quote!(
                        #wc
                        PropView #generics: TryFromState<PropViewEnum>,
                        <PropView #generics as TryFromState<PropViewEnum>>::Error: std::fmt::Debug,
                    )
                });

                quote! {
                    impl #generics FromDependencies for #structure_identity #generics
                        #where_clause
                    {
                        fn from_dependencies(
                            dependencies: &[DependenciesCreatedForDataQuery],
                        ) -> Self {

                            let mut data_struct = #structure_identity::default();
                            let mut mapping_data = #data_identity::default();

                            #(#initialize_data_struct_statements)*

                            data_struct._data_query_mapping_data = mapping_data;

                            data_struct
                        }

                        fn mark_data_viewed(&mut self) {
                            #(#mark_data_viewed_statements)*
                        }

                    }

                    impl #generics #structure_identity #generics
                        #where_clause
                    {
                        /// Return the updates queued during by calls to `queue_update()`
                        /// on the dependencies of this `data` structure.
                        ///
                        /// Returns all queued updates since the last call to `queued_updates()`.
                        ///
                        /// The result of this function is intended to be sent as the return value
                        /// for `invert`.
                        fn queued_updates(&mut self) -> Vec<DependencyValueUpdateRequest> {
                            let mut requests = Vec::new();
                            #(#return_update_requests_statements)*
                            requests
                        }

                    }

                    #[derive(Debug, Default)]
                    struct #data_identity {
                        #(#data_struct_statements)*
                    }

                }
            }
            _ => panic!("only named fields supported"),
        },
        _ => panic!("only structs supported"),
    };
    output.into()
}

pub fn prop_data_queries_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let structure_identity = &ast.ident;
    let data = &ast.data;

    let output = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // eprintln!("named: {:#?}", named);

                let field_identities = named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().clone())
                    .collect::<Vec<_>>();

                let num_queries = field_identities.len();

                let structure_name = structure_identity.to_string();

                let structure_is_data_queries = structure_name.ends_with("DataQueries");

                if structure_is_data_queries {
                    let mut from_structure_to_vec_statements = Vec::new();

                    for field_identity in field_identities.iter() {
                        from_structure_to_vec_statements.push(quote! {
                            structure.#field_identity.as_ref().map(|inst| {
                                instruct_vec.push(inst.clone());
                            });
                        });
                    }
                    quote! {
                        impl From<&#structure_identity> for Vec<DataQuery> {
                            fn from(structure: &#structure_identity) -> Self {
                                let mut instruct_vec = Vec::with_capacity(#num_queries);
                                #(#from_structure_to_vec_statements)*
                                instruct_vec
                            }
                        }
                    }
                } else {
                    let mut data_query_struct_statements = Vec::new();

                    let mut from_structure_to_vec_statements = Vec::new();

                    let data_query_name = format!("{}Queries", structure_name);

                    let data_query_identity = Ident::new(&data_query_name, Span::call_site());

                    for field_identity in field_identities.iter() {
                        if field_identity.to_string().starts_with('_') {
                            continue;
                        }

                        data_query_struct_statements.push(quote! {
                            pub #field_identity: DataQuery,
                        });

                        from_structure_to_vec_statements.push(quote! {
                            instruct_vec.push(structure.#field_identity);
                        });
                    }

                    quote! {
                        #[derive(Debug, Default)]
                        struct #data_query_identity {
                            #(#data_query_struct_statements)*
                        }

                        impl From<#data_query_identity> for Vec<DataQuery> {
                            fn from(structure: #data_query_identity) -> Self {
                                let mut instruct_vec = Vec::with_capacity(#num_queries);
                                #(#from_structure_to_vec_statements)*
                                instruct_vec
                            }
                        }

                    }
                }
            }
            _ => panic!("only named fields supported"),
        },
        _ => panic!("only structs supported"),
    };
    output.into()
}

pub fn add_dependency_data_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast: syn::DeriveInput = syn::parse(item).unwrap();
    let structure_identity = &ast.ident;

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(fields) = &mut struct_data.fields {
                let structure_name = structure_identity.to_string();
                let sn_len = structure_name.len();

                let data_name = if structure_name.ends_with("ies") {
                    format!("{}yData", &structure_name[..(sn_len - 3)])
                } else {
                    format!("{}Data", structure_name)
                };
                let data_identity = Ident::new(&data_name, Span::call_site());

                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            _data_query_mapping_data: #data_identity
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast
            }
            .into()
        }
        _ => panic!("`add_standard_component_fields` has to be used with structs."),
    }
}

#[cfg(test)]
mod test {}
