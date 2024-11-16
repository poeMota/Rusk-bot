use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    parse_macro_input, Expr, ExprArray, FnArg, GenericArgument, Ident, ItemFn, Lit, Pat, PatType,
    PathArguments, Type,
};

// Comments generated by chatgpt, I was lazy.
// To be honest I didn't even read them
#[proc_macro_attribute]
pub fn slash_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse macro attributes and the input function
    let attributes = parse_macro_input!(attr as syn::ExprArray);
    let input_function = parse_macro_input!(item as ItemFn);

    let function_name = &input_function.sig.ident;
    let function_name_str = &input_function.sig.ident.to_string();
    let function_inputs = &input_function.sig.inputs;

    // Generate the locale string for the command name
    let command_locale_key = function_name_str.replace("_", "-").to_lowercase() + "-command";

    // Initialize variables for command generation
    let mut parameter_index: usize = 0;
    let mut command_choices_code = quote! {}; // Code for command choices (like enums)
    let mut parameter_conversion_code = quote! {}; // Code for converting input parameters
    let mut parameter_names = Vec::new(); // List of parameter names

    let mut parameter_options = HashMap::new();
    let mut parameter_choices = HashMap::new();
    let mut parameter_default_values = HashMap::new();

    // Parse macro attributes (like min/max values, choices, etc.)
    for attribute in attributes.elems {
        match attribute {
            Expr::Assign(assign_expr) => {
                // Check if the left-hand side is an identifier (parameter name)
                if let Expr::Path(path_expr) = *assign_expr.left {
                    let param_identifier_token = path_expr.to_token_stream();
                    let param_name = param_identifier_token.to_string();

                    match *assign_expr.right {
                        Expr::Array(ExprArray { elems, .. }) => {
                            let mut param_options_code = quote! {};

                            // Iterate over array elements (parameter configurations)
                            for elem in elems.iter() {
                                match elem {
                                    Expr::Assign(assign) => {
                                        let key = assign.left.to_token_stream().to_string();
                                        let value = assign.right.to_token_stream();

                                        // Handle different macro attributes
                                        match key.as_str() {
                                            "base_value" => {
                                                parameter_default_values.insert(
                                                    param_name.clone(),
                                                    match *assign.right {
                                                        Expr::Lit(ref literal) => match &literal.lit {
                                                            Lit::Int(int_val) => {
                                                                let parsed_value = int_val.base10_parse::<i64>().unwrap();
                                                                quote! { Some(#parsed_value) }
                                                            }
                                                            Lit::Float(float_val) => {
                                                                let parsed_value = float_val.base10_parse::<f64>().unwrap();
                                                                quote! { Some(#parsed_value) }
                                                            }
                                                            _ => quote! { Some(#value.to_string()) }
                                                        },
                                                        _ => panic!("base_value must be a literal"),
                                                    },
                                                );
                                            }
                                            "min_int_value" => {
                                                if let Expr::Lit(ref lit) = *assign.right {
                                                    if let Lit::Int(int_val) = &lit.lit {
                                                        let min_value =
                                                            int_val.base10_parse::<u64>().unwrap();
                                                        param_options_code = quote! {
                                                            #param_options_code
                                                            .min_int_value(#min_value)
                                                        };
                                                    } else {
                                                        panic!("min_int_value must be an integer");
                                                    }
                                                }
                                            }
                                            "max_int_value" => {
                                                if let Expr::Lit(ref lit) = *assign.right {
                                                    if let Lit::Int(int_val) = &lit.lit {
                                                        let max_value =
                                                            int_val.base10_parse::<u64>().unwrap();
                                                        param_options_code = quote! {
                                                            #param_options_code
                                                            .max_int_value(#max_value)
                                                        };
                                                    } else {
                                                        panic!("max_int_value must be an integer");
                                                    }
                                                }
                                            }
                                            "min_length" => {
                                                if let Expr::Lit(ref lit) = *assign.right {
                                                    if let Lit::Int(int_val) = &lit.lit {
                                                        let min_value =
                                                            int_val.base10_parse::<u16>().unwrap();
                                                        param_options_code = quote! {
                                                            #param_options_code
                                                            .min_length(#min_value)
                                                        };
                                                    } else {
                                                        panic!("min_length must be an integer");
                                                    }
                                                }
                                            }
                                            "max_length" => {
                                                if let Expr::Lit(ref lit) = *assign.right {
                                                    if let Lit::Int(int_val) = &lit.lit {
                                                        let min_value =
                                                            int_val.base10_parse::<u16>().unwrap();
                                                        param_options_code = quote! {
                                                            #param_options_code
                                                            .max_length(#min_value)
                                                        };
                                                    } else {
                                                        panic!("max_length must be an integer");
                                                    }
                                                }
                                            }
                                            "choice" => {
                                                let choice_locale_key = format!(
                                                    "{}-param-{}-choice",
                                                    command_locale_key,
                                                    param_name.to_lowercase().replace("_", "-")
                                                );
                                                let choice_name_ident = syn::Ident::new(
                                                    format!("{}_choice", param_name).as_str(),
                                                    proc_macro2::Span::call_site(),
                                                );

                                                // Code to load choices
                                                command_choices_code = quote! {
                                                    #command_choices_code
                                                    let #choice_name_ident: std::collections::HashMap<String, String> =
                                                        get_string(#choice_locale_key, None)
                                                        .trim()
                                                        .split("\n")
                                                        .map(|e| (get_string(e, None), e.to_string()))
                                                        .collect();
                                                };

                                                // Handle different types of choices (int or string)
                                                match value.to_string().as_str() {
                                                    "int" => {
                                                        parameter_choices.insert(
                                                            param_name.clone(),
                                                            (
                                                                choice_name_ident.clone(),
                                                                quote! {
                                                                    add_int_choice(choice, #choice_name_ident
                                                                        .iter()
                                                                        .position(|x| x == choice.as_str())
                                                                        .unwrap() as i32
                                                                    )
                                                                }
                                                            ),
                                                        );
                                                    }
                                                    "locale" => {
                                                        parameter_choices.insert(
                                                            param_name.clone(),
                                                            (
                                                                choice_name_ident.clone(),
                                                                quote! {
                                                                    add_string_choice(name.chars().take(100).collect::<String>().as_str(), value)
                                                                },
                                                            ),
                                                        );
                                                    }
                                                    _ => panic!("Unsupported choice type"),
                                                }
                                            }
                                            _ => {} // Ignore unknown attributes
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            parameter_options.insert(param_name.clone(), param_options_code);
                        }
                        _ => panic!("Command options must be specified as an array"),
                    }
                }
            }
            _ => {}
        }
    }

    // Parse function parameters and generate conversion logic
    let mut command_parameters = Vec::new();
    for input in function_inputs.iter().skip(2) {
        // Skip ctx: Context and inter:
        // CommandInteraction params
        match input {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let param_name = match **pat {
                    Pat::Ident(ref ident) => &ident.ident,
                    _ => panic!("Unsupported parameter pattern"),
                };
                let param_locale_key = param_name.to_string().replace("_", "-").to_lowercase();
                let param_type_str;

                match &**ty {
                    Type::Path(type_path) => {
                        let path = &type_path.path;
                        let param_conversion_code;
                        let mut is_required = true;

                        // Handle Option<T> types (optional parameters)
                        if path.segments.len() == 1 && path.segments[0].ident == "Option" {
                            if let PathArguments::AngleBracketed(args) = &path.segments[0].arguments
                            {
                                if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                    is_required = false;
                                    param_type_str = get_type_name(inner_type)
                                        .expect("Unsupported type inside Option");

                                    param_conversion_code = generate_option_converter(
                                        param_name.to_token_stream().to_string(),
                                        param_type_str,
                                        &inner_type,
                                        parameter_index,
                                        if is_required {
                                            None
                                        } else {
                                            match parameter_default_values
                                                .get(&param_name.to_string())
                                            {
                                                Some(value) => Some(value.clone()),
                                                None => Some(quote! {None}),
                                            }
                                        },
                                    );
                                } else {
                                    panic!("Unsupported type for Option parameter")
                                }
                            } else {
                                panic!("Error parsing Option type")
                            }
                        } else {
                            param_type_str =
                                get_type_name(&ty).expect("Unsupported parameter type");

                            param_conversion_code = generate_option_converter(
                                param_name.to_token_stream().to_string(),
                                param_type_str,
                                &ty,
                                parameter_index,
                                None,
                            );
                        }

                        parameter_index += 1;
                        parameter_conversion_code = quote! {
                            #parameter_conversion_code
                            #param_conversion_code
                        };
                        parameter_names.push(param_name);

                        // Add parameter options (min/max values, choices, etc.)
                        let param_name_str = param_name.to_token_stream().to_string();
                        command_parameters.push(generate_option_token_stream(
                            param_type_str,
                            &command_locale_key,
                            &param_locale_key,
                            is_required,
                            parameter_choices.get(&param_name_str).cloned(),
                            parameter_options.get(&param_name_str).cloned(),
                        ));
                    }
                    _ => panic!("Unsupported parameter type"),
                }
            }
            _ => {}
        }
    }

    // Generate the command declaration
    let command_declaration = quote! {
        #command_choices_code
        guild.create_command(&ctx.http, serenity::builder::CreateCommand::new(
                get_string(format!("{}-name", #command_locale_key).as_str(), None).chars().take(32).collect::<String>().as_str())
                .description(get_string(format!("{}-description", #command_locale_key).as_str(), None).as_str())
                #(#command_parameters)*
        )
        .await
            .expect(format!("Failed to create command {}", #command_locale_key).as_str());
    };

    // Generate the function call with converted parameters
    let function_call_code = quote! {
        std::sync::Arc::new(|command: serenity::model::application::CommandInteraction, ctx: std::sync::Arc<serenity::client::Context>| {
            Box::pin(async move {
                tokio::task::spawn(async move {
                    #parameter_conversion_code
                    #function_name(&ctx, command, #(#parameter_names),*).await;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }).await?
            })
        })
    };

    // Output the final macro result
    let output_code = quote! {
        let command_enabled = match CONFIG.read().await.commands.get(#function_name_str) {
            Some(is_enabled) => *is_enabled,
            None => true,
        };

        if command_enabled {
            use std::sync::Arc;
            #command_declaration

            let mut command_manager = COMMANDMANAGER.write().await;
            command_manager.add_command(
                get_string(format!("{}-name", #command_locale_key).as_str(), None).chars().take(32).collect::<String>().as_str(),
                #function_call_code
            );

        }

        #input_function
    };

    output_code.into()
}

// Generates a token stream for an option within a command.
fn generate_option_token_stream(
    option_type: &str, // The type of the option (e.g., String, Boolean, Integer, etc.)
    command_key: &String, // Command identifier used for localization keys.
    option_key: &String, // The option identifier used for localization keys.
    is_required: bool, // Indicates if the option is required for the command.
    choice_data: Option<(proc_macro2::Ident, proc_macro2::TokenStream)>, // Optional choice data (for enums).
    additional_options: Option<proc_macro2::TokenStream>, // Optional additional configurations (min/max values, etc.)
) -> proc_macro2::TokenStream {
    // Create an identifier for the option type
    let option_type_ident = syn::Ident::new(&option_type, proc_macro2::Span::call_site());

    // Generate the base option token stream
    let mut token_stream;

    // If choices are provided, generate the token stream to fold over the choices
    match choice_data {
        Some((choice_ident, choice_builder)) => {
            token_stream = quote! {
                #choice_ident.iter().fold(serenity::builder::CreateCommandOption::new(
                    serenity::model::application::CommandOptionType::#option_type_ident,
                    get_string(format!("{}-param-{}-name", #command_key, #option_key).as_str(), None).chars().take(32).collect::<String>().as_str(),
                    get_string(format!("{}-param-{}-description", #command_key, #option_key).as_str(), None).as_str(),
                ), |acc, (name, value)| acc.#choice_builder)
                    .required(#is_required)
            }
        }
        // If no choices are provided, generate the option without folding
        None => {
            token_stream = quote! {
                serenity::builder::CreateCommandOption::new(
                    serenity::model::application::CommandOptionType::#option_type_ident,
                    get_string(format!("{}-param-{}-name", #command_key, #option_key).as_str(), None).chars().take(32).collect::<String>().as_str(),
                    get_string(format!("{}-param-{}-description", #command_key, #option_key).as_str(), None).as_str(),
                )
                    .required(#is_required)
            };
        }
    }

    // If additional options (e.g., min/max values) are present, add them to the token stream
    match additional_options {
        Some(options) => {
            token_stream = quote! {
                .add_option(
                    #token_stream
                    #options
                )
            }
        }
        None => {
            token_stream = quote! {
                .add_option(
                    #token_stream
                )
            }
        }
    }

    token_stream
}

// Generates the code to convert a command input into the expected parameter type.
fn generate_option_converter(
    option_name: String,  // The name of the option (parameter).
    option_type: &str,    // The type of the option (e.g., String, Integer).
    resolved_type: &Type, // The actual resolved type in the Rust function.
    option_index: usize,  // The index of the option in the command.
    default_value: Option<proc_macro2::TokenStream>, // An optional default value if the input is missing.
) -> proc_macro2::TokenStream {
    // Create identifiers based on the option type and option name
    let option_type_ident = Ident::new(option_type, proc_macro2::Span::call_site());
    let option_ident = Ident::new(&option_name, proc_macro2::Span::call_site());

    // Prepare prefix/suffix handling for specific types (e.g., String, other primitives)
    let mut suffix = quote! {};
    let mut prefix = quote! {};
    if option_type == "String" {
        suffix = quote! {.clone()}; // For String, we clone the value.
    } else {
        prefix = quote! {*}; // For non-String types, we dereference the value.
    }

    // Generate code for resolving the command option based on its type
    let resolver_code = match get_resolved_type_name(resolved_type) {
        Some(resolved_value_type) => {
            let resolved_type_ident =
                Ident::new(resolved_value_type, proc_macro2::Span::call_site());

            // Generate code that tries to resolve the option value, and fallback to the default if necessary
            let mut code = quote! {
                let #option_ident = command
                    .data
                    .resolved
                    .#resolved_type_ident
                    .get(&#option_ident)
                    .cloned()
            };

            // If a default value is provided, generate additional logic for handling it
            if let Some(_) = default_value {
                code = quote! {
                    let #option_ident = match #option_ident {
                        Some(value) => match command.data.resolved.#resolved_type_ident.get(&value).cloned() {
                            Some(v) => Some(v),
                            None => #default_value,
                        },
                        None => None,
                    };
                };
            } else {
                code = quote! {
                    #code
                    .unwrap()
                    .clone();
                };
            }

            code
        }
        None => {
            // If no resolver is found for the type, use a default fallback mechanism
            if let Some(_) = default_value {
                quote! {
                    let #option_ident = match #option_ident {
                        None => #default_value,
                        Some(value) => Some(value),
                    };
                }
            } else {
                quote! {}
            }
        }
    };

    // Generate the main part of the option conversion logic
    let mut converter_code = quote! {
        let #option_ident = match &command
            .data
            .options
            .get(#option_index)
            .expect("Unexpected error with command option converter")
            .value {
                serenity::model::application::CommandDataOptionValue::#option_type_ident(value) => Some(#prefix value #suffix),
                _ => None,
            }
    };

    // If a default value exists, chain the resolver code
    if let Some(_) = default_value {
        converter_code = quote! {
            #converter_code;
            #resolver_code
        };
    } else {
        converter_code = quote! {
            #converter_code
            .expect("Unsupported ResolvedValue type");
            #resolver_code
        };
    }

    converter_code.into()
}

// Helper function to get the type name of an option (for command options)
fn get_type_name(ty: &Type) -> Option<&'static str> {
    // Match based on the last segment of the path (e.g., String, bool, User, etc.)
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let type_name = path.segments.last()?.ident.to_string();

            match type_name.as_str() {
                "String" => Some("String"),
                "bool" => Some("Boolean"),
                "User" => Some("User"),
                "PartialChannel" => Some("Channel"),
                "Role" => Some("Role"),
                "Attachment" => Some("Attachment"),
                "f64" => Some("Number"),
                "i64" => Some("Integer"),
                _ => None,
            }
        }
        _ => None,
    }
}

// Resolves the appropriate field (users, channels, roles) based on the type for further processing
fn get_resolved_type_name(ty: &Type) -> Option<&'static str> {
    // Match the resolved value type for command data
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let type_name = path.segments.last()?.ident.to_string();

            match type_name.as_str() {
                "String" => None,
                "bool" => None,
                "User" => Some("users"),
                "PartialChannel" => Some("channels"),
                "Role" => Some("roles"),
                "Attachment" => Some("attachments"),
                "f64" => None,
                "i64" => None,
                _ => None,
            }
        }
        _ => None,
    }
}
