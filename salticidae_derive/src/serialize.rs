use crate::helpers::type_to_string;
use proc_macro::TokenStream;
use syn::{export::TokenStream2, Data, DataEnum, Field};

pub fn serialize_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;
    if let Data::Enum(DataEnum { variants, .. }) = &ast.data {
        let serializers = variants_to_serializers(variants.iter(), &name);

        (quote! {
            impl Serializable for #name {
                fn serialize(&self) -> Vec<u8> {
                let mut wtr = vec![];
                match self {
                    #serializers
                };
                wtr
                }
            }
        })
        .into()
    } else {
        TokenStream::new()
    }
}

fn variants_to_serializers(
    variants: syn::punctuated::Iter<'_, syn::Variant>,
    name: &syn::Ident,
) -> TokenStream2 {
    variants
        .map(|varient| -> TokenStream2 { variant_to_serializers(varient, name) })
        .collect::<TokenStream2>()
}

fn variant_to_serializers(variant: &syn::Variant, name: &syn::Ident) -> TokenStream2 {
    let variant_name = &variant.ident;
    let fields = &variant.fields;
    let field_names = fields
        .iter()
        .map(|field| -> &syn::Ident { field.ident.as_ref().unwrap() })
        .collect::<Vec<&syn::Ident>>();
    if let Some((last_field, first_fields)) = fields.iter().collect::<Vec<&Field>>().split_last() {
        let serializers = first_fields
            .iter()
            .map(|field| -> TokenStream2 { field_to_serializer(field, false) })
            .collect::<TokenStream2>();
        let last_serializer = field_to_serializer(last_field, true);
        quote!(
            #name::#variant_name{#(#field_names),*} => {
                #serializers;
                #last_serializer;
            },
        )
    } else {
        quote!(
            #name::#variant_name{} => {},
        )
    }
}
fn field_to_serializer(field: &Field, is_last: bool) -> TokenStream2 {
    match type_to_string(&field.ty).as_ref() {
        "String" => {
            let field_name = &field.ident;
            if is_last {
                quote! {
                    wtr.extend(#field_name.as_bytes().to_vec());
                }
            } else {
                quote! {
                    salticidae::WriteBytesExt::write_u32::<salticidae::LittleEndian>(&mut wtr, #field_name.len() as u32).unwrap();
                    wtr.extend(#field_name.as_bytes().to_vec());
                }
            }
        }
        _ => quote! {},
    }
}
