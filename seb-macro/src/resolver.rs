use proc_macro2::Ident;
use proc_macro2::Span;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    DeriveInput, Field,
};

enum Kind {
    Static(syn::LitStr),
    Dynamic(syn::Ident),
}

impl Kind {
    fn to_string_token(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Static(lit) => quote! { #lit },
            Self::Dynamic(ident) => quote! { self.#ident.as_ref() },
        }
    }
}

pub struct DeriveEntryInput {
    entry_name: Ident,
    kind: Kind,
    fields: Vec<Field>,
}

fn parse_optional_kind_attr(attrs: &[syn::Attribute]) -> Option<syn::LitStr> {
    let kind_attr = find_kind_attr(attrs)?;

    if let syn::Meta::NameValue(name_value) = kind_attr.parse_meta().ok()? {
        if let syn::Lit::Str(lit) = name_value.lit {
            return Some(lit);
        }
    }

    None
}

fn find_kind_attr(attrs: &[syn::Attribute]) -> Option<&syn::Attribute> {
    attrs.iter().find(|attr| {
        attr.path
            .get_ident()
            .map(|ident| *ident == "kind")
            .unwrap_or_default()
    })
}

fn field_has_kind_attr(field: &syn::Field) -> bool {
    find_kind_attr(&field.attrs).is_some()
}

impl Parse for DeriveEntryInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut required = [("cite", false), ("optional", false)];
        let input: DeriveInput = input.parse()?;

        let kind_attr = parse_optional_kind_attr(&input.attrs);

        let mut kind_field = None;

        let fields = match input.data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .into_iter()
                    .filter(|field| {
                        if field_has_kind_attr(field) {
                            kind_field = field.ident.clone();
                            return false;
                        }
                        for (name, seen) in &mut required {
                            if *name
                                == field
                                    .ident
                                    .as_ref()
                                    .map(ToString::to_string)
                                    .unwrap_or_default()
                                    .as_str()
                            {
                                *seen = true;
                                return false;
                            }
                        }
                        true
                    })
                    .collect(),
                syn::Fields::Unit => vec![],
                syn::Fields::Unnamed(_) => panic!("Entry macro does not support tuple structs"),
            },
            _ => panic!("Entry macro is only supported on structs"),
        };

        if !required.iter().all(|(_, f)| *f) {
            panic!(
                "Entry requires a 'cite' and 'optional' field {:?}",
                required
            );
        }

        let kind = if let Some(kind) = kind_attr
            .map(Kind::Static)
            .xor(kind_field.map(Kind::Dynamic))
        {
            kind
        } else {
            // default to using the struct name (lowercase).
            Kind::Static(syn::LitStr::new(
                &input.ident.to_string().to_lowercase(),
                Span::call_site(),
            ))
        };

        Ok(Self {
            kind,
            entry_name: input.ident,
            fields,
        })
    }
}

fn field_name_token_map<'a, F>(
    fields: &'a [syn::Field],
    map: &'a F,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a
where
    F: Fn((&syn::Ident, syn::LitStr)) -> proc_macro2::TokenStream,
{
    fields.iter().map(|f| {
        let name = f
            .ident
            .as_ref()
            .map(|ident| (ident, syn::LitStr::new(&ident.to_string(), ident.span())))
            .unwrap();
        (map)(name)
    })
}

fn match_field_name(fields: &[syn::Field]) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    field_name_token_map(
        fields,
        &|(ident, name)| quote! { #name => Some(&self.#ident), },
    )
}

fn required_field_name_array(fields: &[syn::Field]) -> proc_macro2::TokenStream {
    let names = field_name_token_map(fields, &|(_, name)| quote! { #name });
    quote! { [#(#names,)*] }
}

fn remove_req_fields(fields: &[syn::Field]) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    field_name_token_map(fields, &|(ident, name)| {
        quote! {
            #ident: resolver.fields.remove(#name).unwrap(),
        }
    })
}

impl DeriveEntryInput {
    fn impl_entry_ext_trait(&self) -> proc_macro2::TokenStream {
        let Self {
            kind,
            entry_name,
            fields,
        } = self;

        let get_field_var = match_field_name(fields);

        let field_tuple = fields.iter().map(|f| {
            let ident = &f.ident;
            let name = f
                .ident
                .as_ref()
                .map(|ident| syn::LitStr::new(&ident.to_string(), ident.span()));
            quote! {
                (#name, &self.#ident)
            }
        });

        let kind_global = if let Kind::Static(lit) = kind {
            let global = Ident::new(&entry_name.to_string().to_uppercase(), Span::call_site());
            quote! {
                impl #entry_name {
                    /// A static str that represents the type of the entry.
                    pub const #global: &'static ::std::primitive::str = #lit;
                }
            }
        } else {
            Default::default()
        };

        let kind_str = kind.to_string_token();

        quote! {

            #kind_global

            impl ::seb::ast::EntryExt for #entry_name {
                fn kind(&self) -> &::std::primitive::str {
                    #kind_str
                }

                fn get_field(&self, name: &::std::primitive::str) -> ::std::option::Option<&QuotedString> {
                    match name {
                        #(#get_field_var)*
                        s => self.optional.get(s)
                    }
                }

                fn cite(&self) -> ::std::borrow::Cow<'_, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed(&self.cite)
                }

                fn set_cite(&mut self, cite: ::std::string::String) -> ::std::string::String {
                    ::std::mem::replace(&mut self.cite, cite)
                }

                fn title(&self) -> &QuotedString {
                    &self.title
                }

                fn fields(&self) -> ::std::vec::Vec<::seb::ast::Field<'_>> {
                    let mut fields: ::std::vec::Vec<::seb::ast::Field<'_>> = [#(#field_tuple,)*]
                        .into_iter()
                        .map(::seb::ast::Field::from)
                        .collect();
                    fields.extend(self.optional.iter().map(::seb::ast::Field::from));
                    fields
                }

            }
        }
    }

    fn kind_attr_resolver(
        entry_name: &syn::Ident,
        kind: &syn::LitStr,
        fields: &[syn::Field],
    ) -> proc_macro2::TokenStream {
        let req_field_array = required_field_name_array(fields);
        let remove_req_fields = remove_req_fields(fields);
        quote! {
            impl #entry_name {
                /// Creates a new [`Resolver`] for this type to ensure that the required fields
                /// are set before the entry type can be built.
                ///
                /// Does not set the cite value of the resolver so will be generated based on
                /// the field values.
                pub fn resolver() -> ::seb::ast::Resolver {
                     ::seb::ast::Resolver {
                        kind: ::std::borrow::Cow::Borrowed(#kind),
                        cite: None,
                        req: #req_field_array.into_iter().collect(),
                        fields: ::std::collections::HashMap::default(),
                        entry_resolve: Self::resolve,
                    }

                }

                /// Creates a new [`Resolver`] for this type to ensure that the required fields
                /// are set before the entry type can be built.
                pub fn resolver_with_cite<S>(cite: S) -> ::seb::ast::Resolver
                where
                    S: ::std::convert::Into::<::std::string::String>,
                {
                     ::seb::ast::Resolver {
                        kind: ::std::borrow::Cow::Borrowed(#kind),
                        cite: Some(cite.into()),
                        req: #req_field_array.into_iter().collect(),
                        fields: ::std::collections::HashMap::default(),
                        entry_resolve: Self::resolve,
                    }
                }

                fn resolve(mut resolver: ::seb::ast::Resolver) -> ::std::boxed::Box::<dyn ::seb::ast::EntryExt> {
                    use ::seb::ast::EntryExt;

                    let cite = resolver.cite().to_string();
                    let entry = #entry_name {
                        cite,
                        #(#remove_req_fields)*
                        optional: resolver.fields,
                    };

                    ::std::boxed::Box::new(entry)
                }
            }
        }
    }

    fn kind_field_resolver(
        entry_name: &syn::Ident,
        kind_ident: &syn::Ident,
        fields: &[syn::Field],
    ) -> proc_macro2::TokenStream {
        let req_field_array = required_field_name_array(fields);
        let remove_req_fields = remove_req_fields(fields);
        quote! {
            impl #entry_name {

                /// Creates a new [`Resolver`] for this type to ensure that the required fields
                /// are set before the entry type can be built.
                ///
                /// Does not set the cite value of the resolver so will be generated based on
                /// the field values.
                pub fn resolver<S>(kind: S) -> ::seb::ast::Resolver
                where
                    S: ::std::convert::Into<::std::string::String>,
                {
                    ::seb::ast::Resolver {
                        kind: ::std::borrow::Cow::Owned(kind.into()),
                        cite: None,
                        req: #req_field_array.into_iter().collect(),
                        fields: ::std::collections::HashMap::default(),
                        entry_resolve: Self::resolve,
                    }
                }

                /// Creates a new [`Resolver`] for this type to ensure that the required fields
                /// are set before the entry type can be built.
                pub fn resolver_with_cite<S>(kind: S, cite: S) -> ::seb::ast::Resolver
                where
                    S: ::std::convert::Into<::std::string::String>,
                {
                    ::seb::ast::Resolver {
                        kind: ::std::borrow::Cow::Owned(kind.into()),
                        cite: Some(cite.into()),
                        req: #req_field_array.into_iter().collect(),
                        fields: ::std::collections::HashMap::default(),
                        entry_resolve: Self::resolve,
                    }
                }

                fn resolve(mut resolver: ::seb::ast::Resolver) -> ::std::boxed::Box::<dyn ::seb::ast::EntryExt> {
                    use ::seb::ast::EntryExt;

                    let cite = resolver.cite().to_string();
                    let entry = #entry_name {
                        #kind_ident: resolver.kind,
                        cite,
                        #(#remove_req_fields)*
                        optional: resolver.fields,
                    };

                    ::std::boxed::Box::new(entry)
                }

            }

        }
    }

    fn impl_unit_tests(&self) -> proc_macro2::TokenStream {
        let Self {
            entry_name,
            fields,
            kind,
        } = self;
        let req_fields = required_field_name_array(fields);

        let test_mod_name = format_ident!("{}_tests", entry_name.to_string().to_lowercase());

        let create_resolver = match kind {
            Kind::Static(_) => quote! { #entry_name::resolver_with_cite("old"); },
            Kind::Dynamic(_) => quote! { #entry_name::resolver_with_cite("ignore", "old"); },
        };

        quote! {
            #[cfg(test)]
            mod #test_mod_name {
                use super::#entry_name;
                use ::seb::ast::EntryExt;

                #[test]
                fn resolver_override_cite() {
                    use std::collections::VecDeque;

                    let mut resolver = #create_resolver
                    let mut required: VecDeque<_> = #req_fields.into_iter().collect();

                    let iter = std::iter::from_fn(move || required.pop_front());
                    let iter = iter.zip(('a'..).into_iter());

                       for (field, c) in iter {
                        resolver.set_field(field, c.to_string());
                       }
                    let res = resolver.resolve();
                    let mut entry = res.expect("All required fields added so should have built correctly");

                    assert_eq!("old", entry.cite());
                    entry.set_cite("new".to_owned());
                    assert_eq!("new", entry.cite());
                }

                #[test]
                fn resolver_only_returns_ok_when_all_required_fields_set() {
                    use std::collections::VecDeque;

                    let mut resolver = #create_resolver
                    let mut required: VecDeque<_> = #req_fields.into_iter().collect();

                    let iter = std::iter::from_fn(move || required.pop_front());
                    let iter = iter.zip(('a'..).into_iter());

                    for (field, c) in iter {
                        resolver = resolver
                            .resolve()
                            .expect_err("Resolver should not resolve correctly without required fields");

                        resolver.set_field(field, c.to_string());
                    }
                    resolver.set_field("test", "value");
                    let res = resolver.resolve();
                    let entry = res.expect("All required fields added so should have built correctly");

                    let mut alpha = ('a'..).into_iter().map(|c| c.to_string());

                    for req in #req_fields {
                        let expected = alpha.next().unwrap();
                        let field = entry.get_field(req).unwrap();
                        assert_eq!(expected, field.as_ref());
                    }

                }
            }
        }
    }
}

impl ToTokens for DeriveEntryInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            kind,
            entry_name,
            fields,
        } = self;

        let resolver = match kind {
            Kind::Static(lit) => Self::kind_attr_resolver(entry_name, lit, fields),
            Kind::Dynamic(ident) => Self::kind_field_resolver(entry_name, ident, fields),
        };

        let entry_ext_trait_impl = self.impl_entry_ext_trait();
        let unit_test_impl = self.impl_unit_tests();

        let res = quote! {
            #resolver

            #entry_ext_trait_impl

            #unit_test_impl
        };

        tokens.extend(res);
    }
}
