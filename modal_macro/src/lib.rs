use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Expr, ItemFn};

#[proc_macro_attribute]
pub fn listen_modal(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attribute = parse_macro_input!(attr as syn::Expr);
    let input_function = parse_macro_input!(item as ItemFn);
    let function_name = &input_function.sig.ident;

    let id;

    if let Expr::Lit(assign_expr) = attribute {
        id = assign_expr.to_token_stream();
    } else {
        panic!("modal id must be String");
    }

    let out = quote! {
        let mut command_manager = COMMANDMANAGER.try_write().expect("Cannot lock COMMANDMANAGER for write to add modal call");
        command_manager.add_modal(
            #id,
            std::sync::Arc::new(|modal: serenity::model::application::ModalInteraction, ctx: std::sync::Arc<serenity::client::Context>| {
                Box::pin(async move {
                    tokio::task::spawn(async move {
                        #function_name(&ctx, modal).await;
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    }).await?
                })
            })
        );
        drop(command_manager);

        #input_function
    };

    out.into()
}
