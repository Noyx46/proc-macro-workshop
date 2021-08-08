use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};
use quote::{quote, quote_spanned};
use proc_macro2::{Ident, Span};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if let syn::Data::Struct(ref data) = input.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            // Name of the struct, ex. "Command"
            let ident = input.ident;

            // Name of builder struct, i.e. "CommandBuilder"
            let builder_ident = Ident::new(&format!("{}Builder", ident), Span::call_site());

            let builder_fields = make_builder_fields(fields);
            let method_fields = make_builder_method_fields(fields);
            let setter_methods = make_builder_setters(fields);
            let built = make_build(fields);

            // The tokens
            let expanded = quote!{
                impl #ident {
                    pub fn builder() -> #builder_ident {
                        #builder_ident {
                            #method_fields
                        }
                    }
                }

                pub struct #builder_ident {
                    #builder_fields
                }

                impl #builder_ident {
                    #setter_methods

                    pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                        std::result::Result::Ok(#ident {
                            #built
                        })
                    }
                }
            };

            return proc_macro::TokenStream::from(expanded);
        }
    }
    unimplemented!();
}

fn make_builder_fields(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        let typing = &f.ty;
        quote_spanned! {f.span()=>
            #name: std::option::Option<#typing>
        }
    });
    return quote!{
        #(#recurse),*
    }
}

fn make_builder_method_fields(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {f.span()=>
            #name: std::option::Option::None
        }
    });
    return quote!{
        #(#recurse),*
    }
}

fn make_builder_setters(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        let typing = &f.ty;
        quote_spanned!{f.span()=>
            pub fn #name(&mut self, #name: #typing) -> &mut Self {
                self.#name = std::option::Option::Some(#name);
                self
            }
        }
    });
    return quote!{
        #(#recurse)*
    }
}

fn make_build(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {f.span()=>
            // #name: self.#name.take().ok_or(String::from("Fields not filled"))?
            #name: match self.#name {
                std::option::Option::Some(ref v) => v.clone(),
                std::option::Option::None =>
                    return std::result::Result::Err(String::from("Fields not filled").into()),
            }
        }
    });
    return quote!{
        #(#recurse),*
    }
}
