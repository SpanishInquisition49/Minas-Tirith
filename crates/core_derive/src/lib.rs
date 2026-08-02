use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Fields, GenericArgument,
    PathArguments, Type, parse_macro_input, spanned::Spanned,
};

fn get_core_crate() -> proc_macro2::TokenStream {
    let found = crate_name("minastirith_core")
        .or_else(|_| crate_name("minastirith"))
        .unwrap_or(FoundCrate::Itself);

    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote!(::#ident)
        }
    }
}

/// Derive `Cyclable` for a unit-only enum, cycling through variants in
/// declaration order.
#[proc_macro_derive(Cyclable)]
pub fn derive_cyclable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return syn::Error::new_spanned(ident, "Cyclable can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    if variants.is_empty() {
        return syn::Error::new_spanned(ident, "Cyclable requires at least one variant")
            .to_compile_error()
            .into();
    }

    let mut names = Vec::new();
    for v in &variants {
        if !matches!(v.fields, Fields::Unit) {
            return syn::Error::new_spanned(&v.ident, "Cyclable supports only unit variants")
                .to_compile_error()
                .into();
        }
        names.push(v.ident.clone());
    }
    let n = names.len();

    let next_arms = names.iter().enumerate().map(|(i, name)| {
        let next = &names[(i + 1) % n];
        quote! { Self::#name => Self::#next, }
    });

    let prev_arms = names.iter().enumerate().map(|(i, name)| {
        let prev = &names[(i + n - 1) % n];
        quote! { Self::#name => Self::#prev, }
    });

    let krate = get_core_crate();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #krate::traits::Cyclable for #ident #ty_generics #where_clause {
            fn next(&self) -> Self {
                match self {
                    #( #next_arms )*
                }
            }

            fn prev(&self) -> Self {
                match self {
                    #( #prev_arms )*
                }
            }
        }
    };
    expanded.into()
}

/// Derive `Focusable` for a struct, using the field named in
/// `#[focus(field_name)]` as the focus value.
#[proc_macro_derive(Focusable, attributes(focus))]
pub fn derive_focusable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let focus_field = match find_single_attr_name(&input.attrs, "focus") {
        Ok(name) => name,
        Err(e) => return e.to_compile_error().into(),
    };

    let Some(field_name) = focus_field else {
        return syn::Error::new_spanned(&ident, r"Missing attribute: #[focus(field_name)]")
            .to_compile_error()
            .into();
    };

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return syn::Error::new_spanned(&ident, "Focusable can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let named = match fields {
        Fields::Named(fields) => fields.named,
        _ => {
            return syn::Error::new_spanned(
                &ident,
                "Selectable supports only structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let focus_ty = match named
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|id| id == &field_name))
    {
        Some(field) => {
            let ty = &field.ty;
            quote! {#ty}
        }
        None => {
            return syn::Error::new_spanned(&ident, "items field not found")
                .to_compile_error()
                .into();
        }
    };

    let krate = get_core_crate();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #krate::traits::Focusable for #ident #ty_generics #where_clause
        {
            type Focus = #focus_ty;
            fn current_focus(&self) -> Self::Focus {
                self.#field_name
            }

            fn current_focus_mut(&mut self) -> &mut Self::Focus {
                &mut self.#field_name
            }
        }
    };
    expanded.into()
}

fn find_single_attr_name(
    attrs: &[Attribute],
    attr_name: &str,
) -> Result<Option<Ident>, syn::Error> {
    let mut found = None;
    for attr in attrs {
        if !attr.path().is_ident(attr_name) {
            continue;
        }

        let ident: Ident = attr.parse_args()?;
        if found.is_some() {
            return Err(syn::Error::new(
                attr.span(),
                format!("duplicate #[{attr_name}(...)] attribute"),
            ));
        }
        found = Some(ident);
    }
    Ok(found)
}

/// Derive `Selectable` for a struct, using the fields named in
/// `#[select(items_field, selected_index_field)]`.
#[proc_macro_derive(Selectable, attributes(select))]
pub fn derive_selectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let (items_field, selected_field) = match find_select_attrs(&input.attrs) {
        Ok(pair) => pair,
        Err(e) => return e.to_compile_error().into(),
    };

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return syn::Error::new_spanned(&ident, "Selectable can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let named = match fields {
        Fields::Named(fields) => fields.named,
        _ => {
            return syn::Error::new_spanned(
                &ident,
                "Selectable supports only structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let item_ty = match named
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|id| id == &items_field))
    {
        Some(field) => match extract_vec_inner_type(&field.ty) {
            Ok(ty) => ty,
            Err(e) => return e.to_compile_error().into(),
        },
        None => {
            return syn::Error::new_spanned(&ident, "items field not found")
                .to_compile_error()
                .into();
        }
    };

    let krate = get_core_crate();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #krate::traits::Selectable for #ident #ty_generics #where_clause {
            type Item = #item_ty;

            fn items(&self) -> &[Self::Item] {
                &self.#items_field
            }

            fn selected_index(&self) -> Option<usize> {
                self.#selected_field
            }

            fn selected_index_mut(&mut self) -> &mut Option<usize> {
                &mut self.#selected_field
            }
        }
    };
    expanded.into()
}

fn find_select_attrs(attrs: &[Attribute]) -> Result<(Ident, Ident), syn::Error> {
    let mut found = None;
    for attr in attrs {
        if !attr.path().is_ident("select") {
            continue;
        }

        let args = attr.parse_args_with(|input: &syn::parse::ParseBuffer<'_>| {
            let a = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let b = input.parse()?;
            Ok((a, b))
        })?;

        if found.is_some() {
            return Err(syn::Error::new(
                attr.span(),
                "duplicate #[select(...)] attribute",
            ));
        }
        found = Some(args);
    }
    found.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "missing #[select(items, selected_index)] attribute",
        )
    })
}

fn extract_vec_inner_type(ty: &Type) -> syn::Result<proc_macro2::TokenStream> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(ty, "expected Vec<T>"));
    };

    let segment = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(ty, "expected Vec<T>"))?;

    if segment.ident != "Vec" {
        return Err(syn::Error::new_spanned(ty, "expected Vec<T>"));
    }

    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    else {
        return Err(syn::Error::new_spanned(ty, "expected Vec<T>"));
    };

    let Some(GenericArgument::Type(inner_ty)) = args.first() else {
        return Err(syn::Error::new_spanned(ty, "expected Vec<T>"));
    };

    Ok(quote::quote!(#inner_ty))
}
