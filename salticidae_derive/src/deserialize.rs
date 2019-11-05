use crate::helpers::type_to_string;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{export::TokenStream2, Data, DataEnum, Field, Ident};

pub fn deserialize_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;
    if let Data::Enum(DataEnum { variants, .. }) = &ast.data {
        let deserializers = variants_to_deserializers(variants.iter(), &name);
        (quote! {
            impl Deserializable for #name {
                fn deserialize(bytes: &[u8], message_type: u8) -> Self {
                    let mut rdr = std::io::Cursor::new(bytes);
                    match message_type {
                        #deserializers
                        _ => panic!("unkown type"),
                    }
                }
            }
        })
        .into()
    } else {
        TokenStream::new()
    }
}

fn variants_to_deserializers(
    variants: syn::punctuated::Iter<'_, syn::Variant>,
    name: &syn::Ident,
) -> TokenStream2 {
    variants
        .enumerate()
        .map(|(index, varient)| -> TokenStream2 { variant_to_deserializers(varient, name, index) })
        .collect::<TokenStream2>()
}

fn variant_to_deserializers(
    variant: &syn::Variant,
    name: &syn::Ident,
    index: usize,
) -> TokenStream2 {
    let variant_name = &variant.ident;
    let message_type = index as u8;
    let fields = &variant.fields;
    let field_names = fields
        .iter()
        .map(|field| -> &syn::Ident { field.ident.as_ref().unwrap() })
        .collect::<Vec<&syn::Ident>>();
    if let Some((last_field, first_fields)) = fields.iter().collect::<Vec<&Field>>().split_last() {
        let deserializers = first_fields
            .iter()
            .map(|field| -> TokenStream2 { field_to_deserializer(field, false) })
            .collect::<TokenStream2>();
        let last_deserializer = field_to_deserializer(last_field, true);
        quote!(
            #message_type => {
                #deserializers;
                #last_deserializer;
                #name::#variant_name{
                    #(#field_names),*
                }
            },
        )
    } else {
        quote!(
            #message_type => #name::#variant_name{},
        )
    }
}
fn field_to_deserializer(field: &Field, is_last: bool) -> TokenStream2 {
    match type_to_string(&field.ty).as_ref() {
        "String" => {
            let field_name = &field.ident;
            let field_len = Ident::new(
                &format!("{}_len", field_name.as_ref().unwrap()),
                Span::call_site(),
            );
            let field_bytes = Ident::new(
                &format!("{}_bytes", field_name.as_ref().unwrap()),
                Span::call_site(),
            );
            if is_last {
                quote! {
                    let mut #field_bytes = Default::default();
                    std::io::Read::read_to_end(&mut rdr, &mut #field_bytes).unwrap();
                    let #field_name = std::str::from_utf8(&#field_bytes)
                        .unwrap()
                        .to_string()
                }
            } else {
                quote! {
                    let #field_len = salticidae::ReadBytesExt::read_u32::<salticidae::LittleEndian>(&mut rdr).unwrap();
                    let mut #field_bytes = vec![0u8; (#field_len as usize)];
                    std::io::Read::read_exact(&mut rdr, &mut #field_bytes).unwrap();
                    let #field_name = std::str::from_utf8(&#field_bytes)
                        .unwrap()
                        .to_string()
                }
            }
        }
        _ => quote! {},
    }
}
