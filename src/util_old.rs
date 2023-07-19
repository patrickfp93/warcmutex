use proc_macro2::TokenStream;
use syn::{
    parse2, parse_quote, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput,
    Field, FnArg, ImplItem, ImplItemFn, Receiver, ReturnType, Signature, Stmt,
};

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

pub fn change_block_method(item: ImplItem, name_structure: String) -> ImplItem {
    if let ImplItem::Fn(method) = item {
        let mut sig = method.sig;
        let mut block = method.block;
        let ident = &sig.ident.clone();
        let mut inputs = sig.inputs.clone();
        filter_inputs(&mut inputs);
        //verifica a saída
        block.stmts.clear();
        let possible_self = find_self(&sig);
        let new_statement: syn::Stmt;
        let semicolon = if there_is_output(&sig) {
            quote::quote! {}
        } else {
            quote::quote! {;}
        };
        let inputs = if inputs.len() > 0 {
            quote::quote!(#inputs)
        } else {
            quote::quote!()
        };
        match possible_self {
            Some(receiver) => {
                if let Some(_) = receiver.mutability {
                    let new_code: TokenStream;
                    new_code = quote::quote! {
                        self.base.lock().unwrap().#ident(#inputs)#semicolon
                    };
                    new_statement = convert_to_stmt(new_code).unwrap();
                } else {
                    let new_code: TokenStream;
                    new_code = quote::quote! {
                        self.base.#ident(#inputs)#semicolon
                    };
                    new_statement = convert_to_stmt(new_code).unwrap();
                }
            }
            None => {
                if check_return_type_is_self(&mut sig, name_structure) {
                    let new_code = quote::quote! {
                        std::sync::Arc::new(std::sync::Mutex::(Base::#ident(#inputs)));
                    };
                    new_statement = convert_to_stmt(new_code).unwrap();
                } else {
                    let new_code = quote::quote! {
                        Base::#ident(#inputs)
                    };
                    new_statement = convert_to_stmt(new_code).unwrap();
                }
            }
        }
        block.stmts.push(new_statement);

        let new_method = ImplItemFn {
            attrs: method.attrs,
            vis: method.vis,
            defaultness: method.defaultness,
            sig,
            block,
        };
        return ImplItem::Fn(new_method);
    }
    return item;
}

fn find_self(signature: &Signature) -> Option<Receiver> {
    // Verifica cada argumento na assinatura
    for input in &signature.inputs {
        if let FnArg::Receiver(receiver) = input {
            // Verifica se o argumento é uma referência `&self` ou `&mut self`
            return Some(receiver.clone());
        }
    }
    None
}

fn there_is_output(sig: &Signature) -> bool {
    if let ReturnType::Default = sig.output {
        return false;
    }
    true
}

fn filter_inputs(inputs: &mut Punctuated<FnArg, Comma>) {
    let old_inputs = inputs.clone();
    inputs.clear();
    for arg in old_inputs.iter() {
        if let FnArg::Typed(_) = arg {
            inputs.push(arg.clone());
        }
    }
}

fn convert_to_stmt(token: TokenStream) -> Option<Stmt> {
    match parse2::<Stmt>(token) {
        Ok(stmt) => return Some(stmt),
        Err(err) => {
            eprintln!("Erro ao converter TokenStream em Stmt: {}", err);
            return None;
        }
    }
}

fn check_return_type_is_self(signature: &mut Signature, name_structure: String) -> bool {
    if let ReturnType::Type(_, return_type) = &mut signature.output {
        if let syn::Type::Path(type_path) = return_type.as_mut() {
            if let Some(segment) = type_path.path.segments.iter_mut().last() {
                if segment.ident == "Self" {
                    return true;
                } else if segment.ident == name_structure {
                    segment.ident = parse_quote!(Self);
                    return true;
                }
            }
        }
    }
    false
}
