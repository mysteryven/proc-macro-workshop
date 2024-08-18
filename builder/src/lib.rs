use proc_macro::TokenStream;
use quote::quote;

/// Extracts the inner type of an Option<T> type. Returns None if the type is not an Option.
fn get_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if path.segments.len() == 1 && path.segments[0].ident == "Option" {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
            }) = &path.segments[0].arguments
            {
                if args.len() == 1 {
                    if let syn::GenericArgument::Type(ty) = &args[0] {
                        return Some(ty);
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &input.ident;
    let bname = format!("{}Builder", input.ident);
    let bname = syn::Ident::new(&bname, input.ident.span());

    let data = match input.data {
        syn::Data::Struct(ref data) => data,
        _ => panic!("Only structs are supported"),
    };

    let builder_filelds = data.fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if get_option_inner_type(ty).is_some() {
            return quote! {#name: #ty};
        }

        quote! {#name: Option<#ty>}
    });

    let builder_methods = data.fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if let Some(ty) = get_option_inner_type(ty) {
            return quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            };
        }

        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let init_fields = data.fields.iter().map(|field| {
        let name = &field.ident;
        quote! {
            #name: None
        }
    });

    let build_fields = data.fields.iter().map(|field| {
        let name = &field.ident;

        if get_option_inner_type(&field.ty).is_some() {
            return quote! {
                #name: self.#name.clone()
            };
        }

        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " not set"))?
        }
    });

    quote! {
            pub struct #bname {
                #(#builder_filelds,)*
            }

            impl #bname {
                #(#builder_methods)*
            }

            impl #name {
                pub fn builder() -> #bname {
                    #bname {
                        #(#init_fields,)*
                    }
                }
             }

             impl #bname {
                pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                    Ok(#name {
                        #(#build_fields,)*
                    })
                }
            }
    }
    .into()
}
