use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn event(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        impl Event for #name {
            fn raise(self) {
                let ev_man = EVENTMANAGER.lock().unwrap();
                ev_man.raise_event::<#name>(self);
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        #input

        register_event::<#name>();
    };

    TokenStream::from(expanded)
}
