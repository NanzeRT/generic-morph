#![recursion_limit = "128"]

/// Derives a GenericMut instance based on HList for
/// a given Struct or Tuple Struct
#[proc_macro_derive(GenericMut)]
pub fn generic_mut(input: TokenStream) -> TokenStream {
    // Build the impl
    let gen = impl_generic_mut(input);
    //    println!("{}", gen);
    // Return the generated impl
    gen.into_token_stream().into()
}

use frunk_proc_macro_helpers::*;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::Data;

/// Given an AST, returns an implementation of GenericMut using HList
///
/// Only works with Structs and Tuple Structs
fn impl_generic_mut(input: TokenStream) -> impl ToTokens {
    let ast = to_ast(input);
    let name = &ast.ident;

    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    #[allow(clippy::let_and_return)]
    let tree = match ast.data {
        Data::Struct(ref data) => {
            let field_bindings = FieldBindings::new(&data.fields);
            let repr_type = field_bindings.build_hlist_type(FieldBinding::build_type_mut);
            let hcons_constr = field_bindings.build_hlist_constr(FieldBinding::build);
            let type_constr = field_bindings.build_type_constr(FieldBinding::build);

            quote! {
                #[allow(non_snake_case, non_camel_case_types)]
                impl #impl_generics crate::GenericMut for #name #ty_generics #where_clause {
                    type ReprMut<'_frunk_ref_> = #repr_type;

                    fn into(&mut self) -> Self::ReprMut<'_> {
                        let #name #type_constr = self;
                        #hcons_constr
                    }
                }
            }
        }
        _ => panic!("Only Structs are supported. Enums/Unions cannot be turned into Generics."),
    };

    //     print!("{}", tree);
    tree
}
