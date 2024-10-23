use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Expr, FnArg, GenericArgument, Ident, ItemFn, Lit, Pat, PatType,
    PathArguments, Type,
};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as syn::ExprArray);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_string = &input.sig.ident.to_string();
    let inputs = &input.sig.inputs;

    let fn_locale = fn_name_string.replace("_", "-").to_lowercase() + "-command";

    let mut param_num: usize = 0;
    let mut command_converter = quote! {};
    let mut converter_params = Vec::new();

    let mut params = HashMap::new();

    // Get name and value of macro attributes
    for elem in attrs.elems {
        match elem {
            Expr::Assign(assign) => {
                if let Expr::Path(path) = *assign.left {
                    let ident = path.to_token_stream().to_string();
                    match *assign.right {
                        Expr::Lit(lit) => {
                            params.insert(ident, lit);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Get func params info
    let mut fn_params = Vec::new();
    for arg in inputs.iter() {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let param_name = match **pat {
                    Pat::Ident(ref ident) => &ident.ident,
                    _ => panic!("Unsupported parameter pattern"),
                };
                let param_locale = param_name.to_string().replace("_", "").to_lowercase();
                let param_type: &str;

                match &**ty {
                    Type::Path(type_path) => {
                        let path = &type_path.path;
                        let param_converter;

                        // Fucking hell
                        let mut required = true;

                        if path.segments.len() == 1
                            && path.segments.get(0).unwrap().ident == "Option"
                        {
                            if let PathArguments::AngleBracketed(args) = &path.segments[0].arguments
                            {
                                if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                    required = false;
                                    param_type = get_type_name(&inner_type).expect(
                                        "Unsupported command function param type (in Option)",
                                    );

                                    param_converter = get_option_converter(
                                        param_name.to_token_stream().to_string(),
                                        param_type,
                                        &inner_type,
                                        param_num,
                                        match required {
                                            true => None,
                                            false => Some(quote! {None}),
                                        },
                                    );
                                } else {
                                    panic!("Unsupported command function param type (error with GenericArgument type)")
                                }
                            } else {
                                panic!(
                                    "Unsupported command function param type (error with Option)"
                                )
                            }
                        } else {
                            param_type = get_type_name(&ty).expect(
                                format!(
                                    "Unsupported command function param type: {}",
                                    path.segments.last().unwrap().ident.to_string()
                                )
                                .as_str(),
                            );

                            param_converter = get_option_converter(
                                param_name.to_token_stream().to_string(),
                                param_type,
                                &ty,
                                param_num,
                                None,
                            );
                        }

                        param_num += 1;
                        command_converter = quote! {
                            #command_converter
                            #param_converter
                        };
                        converter_params.push(param_name);

                        fn_params.push(option_token_stream(
                            param_type,
                            &fn_locale,
                            &param_locale,
                            required,
                        ));
                    }
                    _ => {
                        panic!("Unsupported command function param type (type not Path)")
                    }
                }
            }
            _ => {}
        }
    }

    let command_declaration = quote! {
        Box::new(|guild: serenity::model::id::GuildId, ctx: &serenity::client::Context| {
            Box::pin(async move {
                guild.create_command(&ctx.http, serenity::builder::CreateCommand::new("")
                        .name(get_string(format!("{}-name", #fn_locale).as_str(), None).as_str())
                        .description(get_string(format!("{}-description", #fn_locale).as_str(), None).as_str())
                        #(#fn_params)*
                )
                .await
                    .expect(format!("Failed to create command {}", #fn_locale).as_str());
            })
        })
    };

    let call_converter = quote! {
        Box::new(|command: serenity::model::application::CommandInteraction, ctx: serenity::client::Context| {
            Box::pin(async move {
                #command_converter
                #fn_name(#(#converter_params),*);
            })
        })
    };

    let output = quote! {
        let command_enabled = match CONFIG.lock().unwrap().commands.get(#fn_name_string) {
            Some(b) => *b,
            None => true,
        };

        if command_enabled {
            let mut command_manager = COMMANDMANAGER.try_lock().expect("Deadlock on COMMANDMANAGER in command macro :(");
            command_manager.add_command(get_string(format!("{}-name", #fn_locale).as_str(), None).as_str(), #command_declaration, #call_converter);

            #input
        }
    };

    output.into()
}

fn option_token_stream(
    option: &str,
    command_name: &String,
    option_name: &String,
    required: bool,
) -> proc_macro2::TokenStream {
    let type_ident = syn::Ident::new(&option, proc_macro2::Span::call_site());

    quote! {
        .add_option(serenity::builder::CreateCommandOption::new(
            serenity::model::application::CommandOptionType::#type_ident,
            get_string(format!("{}-param-{}-name", #command_name, #option_name).as_str(), None).as_str(),
            get_string(format!("{}-param-{}-description", #command_name, #option_name).as_str(), None).as_str(),
        )
            .required(#required)
        )
    }
}

fn get_option_converter(
    option: String,
    option_type: &str,
    real_option_type: &Type,
    option_number: usize,
    base_value: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let indent = Ident::new(option_type, proc_macro2::Span::call_site());
    let option_name = Ident::new(&option, proc_macro2::Span::call_site());

    let mut suffix = quote! {};
    let mut preffix = quote! {};

    if option_type == "String" {
        suffix = quote! {.clone()}
    } else {
        preffix = quote! {*};
    }

    let converter = match get_resolved_type_name(real_option_type) {
        Some(value) => {
            let suffix_indent = Ident::new(value, proc_macro2::Span::call_site());
            let mut c = quote! {
                let #option_name = command
                    .data
                    .resolved
                    .#suffix_indent
                    .get(& #option_name)
                    .cloned()
            };
            if let Some(_) = base_value {
                c = quote! {
                    let #option_name = match #option_name {
                        Some(value) => {
                            command
                                .data
                                .resolved
                                .#suffix_indent
                                .get(&value)
                                .cloned()
                        },
                        None => None,
                    };
                }
            } else {
                c = quote! {
                    #c
                    .unwrap()
                    .clone();
                }
            }
            c
        }
        None => {
            if let Some(_) = base_value {
                quote! {
                    let #option_name = match #option_name {
                        None => #base_value,
                        Some(value) => Some(value),
                    };
                }
            } else {
                quote! {}
            }
        }
    };

    let mut out: proc_macro2::TokenStream = quote! {
        let #option_name = match &command
            .data
            .options
            .get(#option_number)
            .expect("Unexpected error with command option converter")
            .value {
                serenity::model::application::CommandDataOptionValue::#indent (value) => Some(#preffix value #suffix),
                _ => None,
            }
    }
    .into();

    if let Some(_) = base_value {
        out = quote! {
            #out;
            #converter
        };
    } else {
        out = quote! {
            #out
            .expect("Unsupported ResolvedValue type");
            #converter
        }
    }
    out.into()
}

fn get_type_name(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let ident = path.segments.last()?.ident.to_string();

            match ident.as_str() {
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

fn get_resolved_type_name(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let ident = path.segments.last()?.ident.to_string();

            match ident.as_str() {
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
