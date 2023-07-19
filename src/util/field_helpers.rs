use syn::{DeriveInput, Field, Data, DataStruct};

pub fn extract_fields(input: &DeriveInput) -> Vec<Field> {
    if let Data::Struct(DataStruct { fields, .. }) = &input.data {
        let fields = fields
            .iter()
            .map(|field| field.clone())
            .collect::<Vec<Field>>();

        return fields;
    } else {
        // Retorne um TokenStream vazio se não for uma estrutura
        vec![]
    }
}