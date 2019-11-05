use syn::{Path, PathSegment, Type, TypePath};

pub fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => match segments.first() {
            Some(PathSegment { ident, .. }) => ident.to_string(),
            _ => "".to_string(),
        },
        _ => panic!(format!("unknown type: {}", quote! {ty})),
    }
}
