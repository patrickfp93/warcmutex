use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, FnArg, ImplItem, ImplItemFn, ItemImpl,
    Pat, ReturnType, Signature, Stmt, Type, Visibility,
};

pub fn remove_pub_from_impl_item(item: &mut ImplItem) {
    if let ImplItem::Fn(method) = item {
        if let Visibility::Public(_) = method.vis {
            method.vis = Visibility::Inherited;
        }
    }
}

pub fn change_block_method(original_impl_item: &ImplItem,name_base_type : String) -> Option<ImplItem> {
    let mut clone_impl_item = original_impl_item.clone();
    if let ImplItem::Fn(method) = &mut clone_impl_item {
        if remake_block(method,name_base_type) {
            return Some(clone_impl_item);
        } else {
            return None;
        }
    }
    None
}

fn remake_block(method: &mut ImplItemFn, name_base_type : String ) -> bool {
    let sig = method.sig.clone();
    let reference = if find_self(&sig) {
        "self.base.lock().unwrap().".to_string()
    } else {
        format!("{}::",name_base_type)
    };
    let mut inputs = sig.inputs.clone();
    filter_inputs(&mut inputs);
    let inputs = if sig.inputs.len() > 0 {
        convert_punctuated_to_string(&sig.inputs)
    } else {
        "".into()
    };

    let return_: String = if let ReturnType::Type(_, type_) = sig.output.clone() {
        if let Some(stmts) =
            remake_block_extend(&sig.output, &reference, &sig.ident.to_string(), &inputs)
        {
            method.block.stmts.clear();
            for stmt in stmts {
                method.block.stmts.push(stmt);
            }
            return true;
        } else if let Type::Reference(_) = type_.as_ref() {
            return false;
        }
        "return ".into()
    } else {
        "".into()
    };
    let stmt_str = format!(
        "{}{}{}({});",
        return_,
        reference,
        sig.ident.to_string(),
        inputs,
    );
    println!(
        "Stmt_str do método chamado {} ->>>>>>>>> {}",
        sig.ident, &stmt_str
    );
    let stmt = str_to_stmt(&stmt_str);
    method.block.stmts.clear();
    method.block.stmts.push(stmt);
    true
}

fn find_self(signature: &Signature) -> bool {
    // Verifica cada argumento na assinatura
    for input in &signature.inputs {
        if let FnArg::Receiver(_) = input {
            // Verifica se o argumento é uma referência `&self` ou `&mut self`
            return true;
        }
    }
    false
}

fn convert_punctuated_to_string(args: &Punctuated<FnArg, Comma>) -> String {
    let mut reply = "".to_string();
    for (index, arg) in args.iter().enumerate() {
        if let FnArg::Typed(param) = arg {
            if let Pat::Ident(pat_ident) = &*param.pat {
                let ident = pat_ident.ident.clone();
                reply += &ident.to_string();
                if index < args.len() - 1 {
                    reply += ",";
                }
            }
        }
    }
    reply
}

fn remake_block_extend(
    return_type: &ReturnType,
    reference: &str,
    ident: &str,
    inputs: &str,
) -> Option<Vec<Stmt>> {
    if let ReturnType::Type(_, ty) = return_type {
        match &**ty {
            Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Self" {
                        let mut code = "return Self{base: Arc::new(Mutex::new(".to_string();
                        code.push_str(reference);
                        code.push_str(ident);
                        code.push_str("(");
                        code.push_str(inputs);
                        code.push_str(")))};");
                        return Some(vec![str_to_stmt(&code)]);
                    }
                }
            }
            Type::Reference(type_reference) => {
                if let Type::Path(type_path) = &*type_reference.elem {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "Self" {
                            let code = format!("{}{}({});", reference, ident, inputs);
                            let mut reply = vec![];
                            reply.push(str_to_stmt(&code));
                            reply.push(str_to_stmt("return self;"));
                            return Some(reply);
                        }
                    }
                }
            }
            _ => {
                return None;
            }
        }
    }
    return None;
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

fn str_to_stmt(code: &str) -> Stmt {
    syn::parse_str::<syn::Stmt>(code).expect(&format!(
        "Failure in the construction of statement:\n{}",
        code
    ))
}

pub fn transform_method_return_type(item_impl: &mut ImplItem,original_struct_name : String) {
    if let syn::ImplItem::Fn(method) = item_impl {
        if let ReturnType::Type(_, type_) = &mut method.sig.output {
            if let Type::Path(type_path) = &**type_{
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == &original_struct_name {
                        method.sig.output = parse_quote! { -> Self };
                    }
                }
            }
        }
    }
}

pub fn get_impl_type_name(item_impl: &ItemImpl) -> Option<String> {
    if let Some((_, path, _)) = &item_impl.trait_ {
        if let Some(segment) = path.segments.last() {
            return Some(segment.ident.to_string());
        }
    } else if let Type::Path(type_path) = &*item_impl.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    
    None
}



