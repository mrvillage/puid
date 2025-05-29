use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    Ident,
    LitStr,
    Result,
};

struct Input {
    name:   syn::Ident,
    prefix: String,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let prefix: LitStr = input.parse()?;
        Ok(Input {
            name,
            prefix: prefix.value(),
        })
    }
}

#[proc_macro]
/// Generates a type for a Prefixed Unique Identifier (PUID).
///
/// # Example:
/// ```
/// puid!(UserId = "usr");
/// let user_id = UserId::new();
/// assert_eq!(user_id.as_str().len(), 26); // "usr_" + _ + 22 base62 characters
/// assert!(user_id.as_str().starts_with("usr_"));
/// ```
pub fn puid(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_puid(parse_macro_input!(input as Input))
        .unwrap()
        .into()
}

fn impl_puid(Input { name, prefix }: Input) -> Result<TokenStream> {
    // 22 bytes for the suffix and 1 byte for the underscore
    let prefix_len = prefix.len();
    let len = prefix_len + 22 + 1;
    let mut buf = Vec::with_capacity(len);
    for i in prefix.bytes() {
        buf.push(i);
    }
    buf.push(b'_');
    buf.resize(len, b'0');

    let serde = if cfg!(feature = "serde") {
        let visitor_ident = syn::Ident::new(&format!("{name}SerdeVisitor"), name.span());
        quote! {
            impl ::serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    serializer.serialize_str(self.as_str())
                }
            }

            struct #visitor_ident;

            impl ::serde::de::Visitor<'_> for #visitor_ident {
                type Value = #name;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("a string with the format '#prefix_<suffix>'")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    v.parse().map_err(E::custom)
                }
            }

            impl<'de> ::serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str(#visitor_ident)
                }
            }
        }
    } else {
        quote! {}
    };

    let snake_case_name = {
        let mut name_str = name.to_string();
        for c in 'A'..='Z' {
            name_str = name_str.replace(c, &format!("_{}", c.to_ascii_lowercase()));
        }
        name_str.trim_start_matches('_').to_string()
    };
    let postgres = if cfg!(feature = "postgres") {
        quote! {
            impl ::sqlx::Type<::sqlx::Postgres> for #name {
                fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                    ::sqlx::postgres::PgTypeInfo::with_name(#snake_case_name)
                }

                fn compatible(ty: &::sqlx::postgres::PgTypeInfo) -> bool {
                    ty == &::sqlx::postgres::PgTypeInfo::with_name("user_id")
                }
            }

            impl ::sqlx::Encode<'_, ::sqlx::Postgres> for #name {
                fn encode_by_ref(&self, buf: &mut ::sqlx::postgres::PgArgumentBuffer) -> ::std::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                    buf.extend(self.as_str().as_bytes());
                    Ok(::sqlx::encode::IsNull::No)
                }
            }

            impl<'r> ::sqlx::Decode<'r, ::sqlx::Postgres> for #name {
                fn decode(value: ::sqlx::postgres::PgValueRef<'r>) -> ::std::result::Result<Self, ::sqlx::error::BoxDynError> {
                    let s: &str = value.as_str()?;
                    s.parse().map_err(::std::convert::Into::into)
                }
            }
        }
    } else {
        quote! {}
    };
    let create_domain = format!(
        "CREATE DOMAIN {snake_case_name} AS CHAR({len}) CHECK (VALUE ~ \
         '^{prefix}_[0-9A-Za-z]{{22}}$');",
    );

    Ok(quote! {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct #name([u8; #len]);

        impl #name {
            /// Creates a new `#name` with the given suffix.
            #[allow(clippy::new_without_default)]
            pub fn new() -> Self {
                let mut buf = [#(#buf),*];

                ::puid::encode_suffix(&mut buf[#prefix_len + 1..]);
                #name(buf)
            }

            pub fn as_str(&self) -> &str {
                unsafe {
                    ::std::str::from_utf8_unchecked(&self.0)
                }
            }

            pub fn create_domain() -> &'static str {
                #create_domain
            }
        }

        impl ::std::str::FromStr for #name {
            type Err = ::puid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() != #len {
                    return Err(::puid::Error::InvalidLength);
                }
                let mut buf = [#(#buf),*];
                // ensure the prefix matches
                if !s.starts_with(#prefix) {
                    return Err(::puid::Error::InvalidPrefix);
                }
                // ensure the next byte is an underscore
                if s.as_bytes()[#prefix_len] != b'_' {
                    return Err(::puid::Error::InvalidFormat);
                }
                // ensure the suffix is valid then copy
                for c in &s.as_bytes()[#prefix_len + 1..] {
                    if !::puid::is_valid_suffix_byte(*c) {
                        return Err(::puid::Error::InvalidSuffixChar(*c));
                    }
                }
                buf[#prefix_len + 1..].copy_from_slice(&s.as_bytes()[#prefix_len + 1..]);
                Ok(#name(buf))
            }
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    .field("value", &self.as_str())
                    .finish()
            }
        }

        #serde

        #postgres
    })
}
