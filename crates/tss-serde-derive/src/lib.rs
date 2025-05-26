use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

/// Derive macro for TSS serialization
#[proc_macro_derive(TssSerialize, attributes(tss))]
pub fn derive_tss_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_serialize_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for TSS deserialization
#[proc_macro_derive(TssDeserialize, attributes(tss))]
pub fn derive_tss_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_deserialize_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_serialize_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let fields = extract_fields(input)?;

    let serialize_fields = fields
        .iter()
        .map(|(field_name, field_type)| generate_field_serialize(field_name, field_type))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        impl ::tss_serde::TssSerialize for #name {
            fn to_tss_bytes(&self) -> Vec<u8> {
                let mut buffer = Vec::new();
                #(#serialize_fields)*
                buffer
            }
        }
    })
}

fn generate_deserialize_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let fields = extract_fields(input)?;

    let deserialize_fields = fields
        .iter()
        .enumerate()
        .map(|(i, (_field_name, field_type))| {
            let value_var = format_ident!("value_{}", i);
            let deserialize_logic = generate_field_deserialize(field_type)?;

            Ok(quote! {
                let #value_var = #deserialize_logic;
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let field_names: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, (field_name, _))| {
            let value_var = format_ident!("value_{}", i);
            quote! { #field_name: #value_var }
        })
        .collect();

    Ok(quote! {
        impl ::tss_serde::TssDeserialize for #name {
            fn from_tss_reader(reader: &mut ::tss_serde::TssReader) -> Result<Self, ::tss_serde::TssError> {
                #(#deserialize_fields)*

                Ok(Self {
                    #(#field_names),*
                })
            }
        }
    })
}

fn extract_fields(input: &DeriveInput) -> syn::Result<Vec<(Ident, Type)>> {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap().clone();
                    let ty = field.ty.clone();
                    Ok((name, ty))
                })
                .collect(),
            _ => Err(syn::Error::new_spanned(
                input,
                "Only named fields are supported",
            )),
        },
        _ => Err(syn::Error::new_spanned(input, "Only structs are supported")),
    }
}

fn generate_field_serialize(field_name: &Ident, field_type: &Type) -> syn::Result<TokenStream2> {
    let serialize_logic = match type_to_string(field_type).as_str() {
        "u8" => quote! { buffer.push(self.#field_name); },
        "u16" => quote! { buffer.extend_from_slice(&self.#field_name.to_be_bytes()); },
        "u32" => quote! { buffer.extend_from_slice(&self.#field_name.to_be_bytes()); },
        "u64" => quote! { buffer.extend_from_slice(&self.#field_name.to_be_bytes()); },
        "i8" => quote! { buffer.push(self.#field_name as u8); },
        "i16" => quote! { buffer.extend_from_slice(&self.#field_name.to_be_bytes()); },
        "i32" => quote! { buffer.extend_from_slice(&self.#field_name.to_be_bytes()); },
        "i64" => quote! { buffer.extend_from_slice(&self.#field_name.to_be_bytes()); },
        ty if ty.starts_with("[u8;") => {
            quote! { buffer.extend_from_slice(&self.#field_name); }
        }
        ty if ty.starts_with("[") && ty.contains("u16") => {
            quote! {
                for item in &self.#field_name {
                    buffer.extend_from_slice(&item.to_be_bytes());
                }
            }
        }
        ty if ty.starts_with("[") && ty.contains("u32") => {
            quote! {
                for item in &self.#field_name {
                    buffer.extend_from_slice(&item.to_be_bytes());
                }
            }
        }
        _ => {
            quote! {
                buffer.extend_from_slice(&::tss_serde::TssSerialize::to_tss_bytes(&self.#field_name));
            }
        }
    };

    Ok(serialize_logic)
}

fn generate_field_deserialize(field_type: &Type) -> syn::Result<TokenStream2> {
    let deserialize_logic = match type_to_string(field_type).as_str() {
        "u8" => quote! {
            ::tss_serde::TssDeserialize::from_tss_reader(reader)?
        },
        "u16" => quote! {
            ::tss_serde::TssDeserialize::from_tss_reader(reader)?
        },
        "u32" => quote! {
            ::tss_serde::TssDeserialize::from_tss_reader(reader)?
        },
        "u64" => quote! {
            ::tss_serde::TssDeserialize::from_tss_reader(reader)?
        },
        "i8" => quote! {
            reader.read_u8()? as i8
        },
        "i16" => quote! {
            {
                let bytes = reader.read_array::<2>()?;
                i16::from_be_bytes(bytes)
            }
        },
        "i32" => quote! {
            {
                let bytes = reader.read_array::<4>()?;
                i32::from_be_bytes(bytes)
            }
        },
        "i64" => quote! {
            {
                let bytes = reader.read_array::<8>()?;
                i64::from_be_bytes(bytes)
            }
        },
        ty if ty.starts_with("[u8;") => {
            let size = extract_array_size(ty)?;
            quote! {
                reader.read_array::<#size>()?
            }
        }
        ty if ty.starts_with("[") && ty.contains("u16") => {
            let count = extract_array_size(ty)?;
            quote! {
                {
                    let mut array = [0u16; #count];
                    for i in 0..#count {
                        array[i] = ::tss_serde::TssDeserialize::from_tss_reader(reader)?;
                    }
                    array
                }
            }
        }
        ty if ty.starts_with("[") && ty.contains("u32") => {
            let count = extract_array_size(ty)?;
            quote! {
                {
                    let mut array = [0u32; #count];
                    for i in 0..#count {
                        array[i] = ::tss_serde::TssDeserialize::from_tss_reader(reader)?;
                    }
                    array
                }
            }
        }
        _ => {
            quote! {
                ::tss_serde::TssDeserialize::from_tss_reader(reader)?
            }
        }
    };

    Ok(deserialize_logic)
}

fn type_to_string(ty: &Type) -> String {
    quote! { #ty }.to_string().replace(" ", "")
}

fn extract_array_size(type_str: &str) -> syn::Result<usize> {
    if let Some(start) = type_str.find(';') {
        if let Some(end) = type_str.find(']') {
            let size_str = &type_str[start + 1..end];
            size_str
                .parse()
                .map_err(|_| syn::Error::new(proc_macro2::Span::call_site(), "Invalid array size"))
        } else {
            Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Invalid array syntax",
            ))
        }
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Not a fixed-size array",
        ))
    }
}
