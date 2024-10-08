use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Event)]
pub fn event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
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
    };

    TokenStream::from(expanded)
}
