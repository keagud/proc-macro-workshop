use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::ParseStream, parse_macro_input, Data, DeriveInput, Fields, FieldsNamed};

//
// Before moving on, have the macro also generate:
//
//     pub struct CommandBuilder {
//         executable: Option<String>,
//         args: Option<Vec<String>>,
//         env: Option<Vec<String>>,
//         current_dir: Option<String>,
//     }
//
// and in the `builder` function:
//
//     impl Command {
//         pub fn builder() -> CommandBuilder {
//             CommandBuilder {
//                 executable: None,
//                 args: None,
//                 env: None,
//                 current_dir: None,
//             }
//         }
//     }
//
//
macro_rules! compile_error {
    ($span: expr, $($arg: expr)*) => {
        syn::Error::new($span, format!($($arg)*))
            .to_compile_error()
            .into()
    };
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let builder_ident = format_ident!("{}Builder", name);

    let Data::Struct(data_struct) = &ast.data else {
        return syn::Error::new(name.span(), "Expected a struct")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data_struct.fields else {
        return syn::Error::new(name.span(), "Expected a struct with named fields")
            .to_compile_error()
            .into();
    };

    let field_names: Vec<Ident> = fields
        .to_owned()
        .named
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect();

    let mut field_methods = vec![];

    fields.named.iter().for_each(|f| {
        let field_name = f.clone().ident.unwrap();
        let field_type = f.clone().ty;

        let token = quote! {

            pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        };

        field_methods.push(token);
    });

    quote! {

        pub struct #builder_ident {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #builder_ident {

            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {

                #( if self.#field_names.is_none() {
                        return Err("Can't build unless all fields are populated".into());
                    })*

                Ok(
                #name {
                    #( #field_names: self.#field_names.clone().unwrap().clone()),*
                })




            }


            #(#field_methods)*
        }

        impl #name {

            pub fn builder() -> #builder_ident {
                    #builder_ident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                    }
            }




        }



    }
    .into()
}
