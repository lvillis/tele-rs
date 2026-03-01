use std::collections::HashMap;

use proc_macro::TokenStream;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, LitStr, Meta, PathArguments, Token, Type, Variant,
    parse_macro_input,
};

#[proc_macro_derive(BotCommands, attributes(command))]
pub fn derive_bot_commands(input: TokenStream) -> TokenStream {
    match derive_bot_commands_impl(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn derive_bot_commands_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let enum_name = input.ident;
    let generics = input.generics;
    let tele_path = tele_crate_path();

    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                enum_name,
                "BotCommands can only be derived for enums",
            ));
        }
    };

    let mut parse_arms = Vec::new();
    let mut description_entries = Vec::new();
    let mut known_command_variants = HashMap::<String, String>::new();

    for variant in data.variants {
        let attrs = parse_variant_attrs(&variant)?;
        let variant_ident = variant.ident.clone();
        let command_name = attrs
            .rename
            .unwrap_or_else(|| to_snake_case(&variant_ident.to_string()));
        validate_command_name(&command_name, &variant_ident)?;
        let description = attrs
            .description
            .unwrap_or_else(|| format!("{command_name} command"));
        validate_command_description(&description, &variant_ident)?;

        let mut parse_names = Vec::with_capacity(1 + attrs.aliases.len());
        parse_names.push(command_name.clone());
        parse_names.extend(attrs.aliases);
        for parse_name in &parse_names {
            validate_command_name(parse_name, &variant_ident)?;
            if let Some(existing_variant) = known_command_variants.get(parse_name) {
                return Err(syn::Error::new_spanned(
                    &variant_ident,
                    format!(
                        "command name `{parse_name}` for variant `{variant_ident}` conflicts with variant `{existing_variant}`"
                    ),
                ));
            }

            known_command_variants.insert(parse_name.clone(), variant_ident.to_string());
        }

        let name_lit = LitStr::new(&command_name, variant_ident.span());
        let desc_lit = LitStr::new(&description, variant_ident.span());

        description_entries.push(quote! {
            #tele_path::bot::CommandDescription {
                command: #name_lit,
                description: #desc_lit,
            }
        });

        let parse_arm = parse_arm_for_variant(&enum_name, &variant_ident, &variant, &tele_path)?;
        for parse_name in parse_names {
            let parse_name_lit = LitStr::new(&parse_name, variant_ident.span());
            let parse_arm_tokens = parse_arm.clone();
            parse_arms.push(quote! {
                #parse_name_lit => #parse_arm_tokens
            });
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #tele_path::bot::BotCommands for #enum_name #ty_generics #where_clause {
            fn parse(command: &str, args: &str) -> Option<Self> {
                let args = args.trim();
                match command {
                    #(#parse_arms,)*
                    _ => None,
                }
            }

            fn descriptions() -> &'static [#tele_path::bot::CommandDescription] {
                &[
                    #(#description_entries),*
                ]
            }
        }
    })
}

fn parse_arm_for_variant(
    enum_name: &syn::Ident,
    variant_ident: &syn::Ident,
    variant: &Variant,
    tele_path: &TokenStream2,
) -> syn::Result<TokenStream2> {
    match &variant.fields {
        Fields::Unit => Ok(quote! {
            if args.is_empty() {
                Some(#enum_name::#variant_ident)
            } else {
                None
            }
        }),
        Fields::Unnamed(fields) => {
            if fields.unnamed.is_empty() {
                return Err(syn::Error::new_spanned(
                    fields,
                    "tuple command variants must have at least one field",
                ));
            }

            let mut value_bindings = Vec::new();
            let mut value_names = Vec::new();
            let field_count = fields.unnamed.len();

            for (index, field) in fields.unnamed.iter().enumerate() {
                let value_ident = format_ident!("__arg_{index}");
                let is_last = index + 1 == field_count;
                let ty = &field.ty;
                validate_field_type(ty, field)?;
                let value_expr = parse_value_expr(ty, is_last);

                value_bindings.push(quote! {
                    let #value_ident: #ty = #value_expr;
                });
                value_names.push(value_ident);
            }

            Ok(quote! {
                {
                    let __tokens = #tele_path::bot::tokenize_command_args(args)?;
                    let mut __cursor: usize = 0;
                    #(#value_bindings)*

                    if __cursor < __tokens.len() {
                        None
                    } else {
                        Some(#enum_name::#variant_ident(#(#value_names),*))
                    }
                }
            })
        }
        Fields::Named(fields) => {
            if fields.named.is_empty() {
                return Err(syn::Error::new_spanned(
                    fields,
                    "named command variants must have at least one field",
                ));
            }

            let mut value_bindings = Vec::new();
            let mut field_assignments = Vec::new();
            let field_count = fields.named.len();

            for (index, field) in fields.named.iter().enumerate() {
                let value_ident = format_ident!("__arg_{index}");
                let field_ident = field.ident.clone().ok_or_else(|| {
                    syn::Error::new_spanned(field, "named field missing identifier")
                })?;
                let is_last = index + 1 == field_count;
                let ty = &field.ty;
                validate_field_type(ty, field)?;
                let value_expr = parse_value_expr(ty, is_last);

                value_bindings.push(quote! {
                    let #value_ident: #ty = #value_expr;
                });
                field_assignments.push(quote! {
                    #field_ident: #value_ident
                });
            }

            Ok(quote! {
                {
                    let __tokens = #tele_path::bot::tokenize_command_args(args)?;
                    let mut __cursor: usize = 0;
                    #(#value_bindings)*

                    if __cursor < __tokens.len() {
                        None
                    } else {
                        Some(#enum_name::#variant_ident { #(#field_assignments),* })
                    }
                }
            })
        }
    }
}

fn parse_value_expr(ty: &Type, is_last: bool) -> TokenStream2 {
    if is_string_type(ty) {
        if is_last {
            return quote! {
                if __cursor >= __tokens.len() {
                    String::new()
                } else {
                    let value = __tokens[__cursor..].join(" ");
                    __cursor = __tokens.len();
                    value
                }
            };
        }

        return quote! {
            {
                let token = match __tokens.get(__cursor) {
                    Some(token) => token,
                    None => return None,
                };
                __cursor += 1;
                token.clone()
            }
        };
    }

    if let Some(inner) = option_inner_type(ty) {
        if is_string_type(inner) {
            if is_last {
                return quote! {
                    if __cursor >= __tokens.len() {
                        None
                    } else {
                        let value = __tokens[__cursor..].join(" ");
                        __cursor = __tokens.len();
                        Some(value)
                    }
                };
            }

            return quote! {
                if __cursor >= __tokens.len() {
                    None
                } else {
                    let token = __tokens[__cursor].clone();
                    __cursor += 1;
                    Some(token)
                }
            };
        }

        return quote! {
            if __cursor >= __tokens.len() {
                None
            } else {
                let token = &__tokens[__cursor];
                __cursor += 1;
                Some(token.parse::<#inner>().ok()?)
            }
        };
    }

    quote! {
        {
            let token = match __tokens.get(__cursor) {
                Some(token) => token,
                None => return None,
            };
            __cursor += 1;
            token.parse::<#ty>().ok()?
        }
    }
}

#[derive(Default)]
struct VariantAttrs {
    rename: Option<String>,
    description: Option<String>,
    aliases: Vec<String>,
}

fn parse_variant_attrs(variant: &Variant) -> syn::Result<VariantAttrs> {
    let mut parsed = VariantAttrs::default();

    for attr in &variant.attrs {
        if !attr.path().is_ident("command") {
            continue;
        }

        let nested: Punctuated<Meta, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for meta in nested {
            match meta {
                Meta::NameValue(name_value) if name_value.path.is_ident("rename") => {
                    let literal: LitStr = syn::parse2(name_value.value.into_token_stream())?;
                    let value = literal.value();
                    if parsed.rename.replace(value).is_some() {
                        return Err(syn::Error::new_spanned(
                            name_value.path,
                            "duplicate `rename` attribute",
                        ));
                    }
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("description") => {
                    let literal: LitStr = syn::parse2(name_value.value.into_token_stream())?;
                    let value = literal.value();
                    if parsed.description.replace(value).is_some() {
                        return Err(syn::Error::new_spanned(
                            name_value.path,
                            "duplicate `description` attribute",
                        ));
                    }
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("alias") => {
                    let literal: LitStr = syn::parse2(name_value.value.into_token_stream())?;
                    parsed.aliases.push(literal.value());
                }
                Meta::List(list) if list.path.is_ident("aliases") => {
                    let aliases: Punctuated<LitStr, Token![,]> =
                        list.parse_args_with(Punctuated::parse_terminated)?;
                    if aliases.is_empty() {
                        return Err(syn::Error::new_spanned(
                            list.path,
                            "`aliases(...)` requires at least one alias",
                        ));
                    }
                    parsed
                        .aliases
                        .extend(aliases.into_iter().map(|alias| alias.value()));
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unsupported command attribute, expected `rename = \"...\"`, `description = \"...\"`, `alias = \"...\"`, or `aliases(\"...\", ...)`",
                    ));
                }
            }
        }
    }

    Ok(parsed)
}

fn validate_command_name(name: &str, span: &impl ToTokens) -> syn::Result<()> {
    if name.is_empty() {
        return Err(syn::Error::new_spanned(
            span,
            "command name cannot be empty",
        ));
    }

    if name.len() > 32 {
        return Err(syn::Error::new_spanned(
            span,
            format!("command name `{name}` exceeds Telegram max length of 32"),
        ));
    }

    let mut chars = name.chars();
    let Some(first_char) = chars.next() else {
        return Err(syn::Error::new_spanned(
            span,
            "command name cannot be empty",
        ));
    };

    if !first_char.is_ascii_lowercase() {
        return Err(syn::Error::new_spanned(
            span,
            format!("command name `{name}` must start with a lowercase ASCII letter"),
        ));
    }

    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(syn::Error::new_spanned(
            span,
            format!(
                "command name `{name}` contains invalid characters; use lowercase ASCII letters, digits, and `_`"
            ),
        ));
    }

    Ok(())
}

fn validate_command_description(description: &str, span: &impl ToTokens) -> syn::Result<()> {
    if description.is_empty() {
        return Err(syn::Error::new_spanned(
            span,
            "command description cannot be empty",
        ));
    }

    if description.len() > 256 {
        return Err(syn::Error::new_spanned(
            span,
            format!("command description exceeds Telegram max length of 256: `{description}`"),
        ));
    }

    Ok(())
}

fn validate_field_type(ty: &Type, span: &impl ToTokens) -> syn::Result<()> {
    if matches!(ty, Type::Reference(_)) {
        return Err(syn::Error::new_spanned(
            span,
            "borrowed command argument types are unsupported; use owned types like `String`",
        ));
    }

    if let Some(inner) = option_inner_type(ty)
        && matches!(inner, Type::Reference(_))
    {
        return Err(syn::Error::new_spanned(
            span,
            "borrowed command argument types inside `Option` are unsupported; use `Option<String>`",
        ));
    }

    Ok(())
}

fn is_string_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "String"),
        _ => false,
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let type_path = match ty {
        Type::Path(type_path) => type_path,
        _ => return None,
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let args = match &segment.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    if args.args.len() != 1 {
        return None;
    }

    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = name.chars().collect();

    for (index, ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
                let prev = chars[index - 1];
                let next = chars.get(index + 1).copied();
                if prev.is_lowercase() || next.is_some_and(|c| c.is_lowercase()) {
                    result.push('_');
                }
            }

            for lower in ch.to_lowercase() {
                result.push(lower);
            }
        } else {
            result.push(*ch);
        }
    }

    result
}

fn tele_crate_path() -> TokenStream2 {
    match crate_name("tele") {
        Ok(FoundCrate::Itself) => quote!(::tele),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            quote!(::#ident)
        }
        Err(_) => quote!(::tele),
    }
}
