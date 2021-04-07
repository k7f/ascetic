#![feature(extend_one)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;

fn parse_string(lit: syn::Lit, span: Span, field: &str) -> Result<String, syn::Error> {
    match lit {
        syn::Lit::Str(s) => Ok(s.value()),
        syn::Lit::Verbatim(s) => Ok(s.to_string()),
        _ => Err(syn::Error::new(span, format!("Failed to parse `{}` as string.", field))),
    }
}

fn parse_arg(arg: syn::NestedMeta) -> Result<(&'static str, String), syn::Error> {
    match arg {
        syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue)) => {
            let ident = namevalue.path.get_ident();

            if let Some(ident) = ident {
                match ident.to_string().to_lowercase().as_str() {
                    "tag" => {
                        let value = parse_string(namevalue.lit.clone(), namevalue.span(), "tag")?;

                        Ok(("tag", value))
                    }
                    "group" => {
                        let value = parse_string(namevalue.lit.clone(), namevalue.span(), "group")?;

                        Ok(("group", value))
                    }
                    name => {
                        let msg = format!(
                            "Unknown attribute `{}` is specified; expected one of: `tag`, `group`",
                            name
                        );
                        Err(syn::Error::new_spanned(namevalue, msg))
                    }
                }
            } else {
                Err(syn::Error::new_spanned(namevalue, "Must have specified ident"))
            }
        }
        other => Err(syn::Error::new_spanned(other, "Unknown attribute inside the macro")),
    }
}

fn parse(args: syn::AttributeArgs, item: syn::ItemMod) -> Result<TokenStream, syn::Error> {
    let attrs = &item.attrs;
    let vis = item.vis;
    let mod_token = item.mod_token;
    let ident = item.ident;
    let mut groups = Vec::new();
    let mut tags = Vec::new();
    let mut body = quote! {};

    for arg in args {
        let (key, value) = parse_arg(arg)?;
        match key {
            "group" => groups.push(value),
            "tag" => tags.push(value),
            _ => unreachable!(),
        }
    }

    for group in groups {
        for tag in tags.iter() {
            let path = format!("/{}_{}.rs", group, tag);

            body.extend_one(quote! {
                include!(concat!(env!("OUT_DIR"), #path));
            });
        }
    }

    if let Some((_, ref content)) = item.content {
        body.extend_one(quote! {
            #(#content)*
        });
    }

    let result = quote! {
        #(#attrs)*
        #vis #mod_token #ident {
            #body
        }
    };

    Ok(result.into())
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn assets(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let item = syn::parse_macro_input!(item as syn::ItemMod);

    parse(args, item).unwrap_or_else(|e| e.to_compile_error().into())
}
