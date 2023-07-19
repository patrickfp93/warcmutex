use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput, ImplItem, ItemImpl, Visibility, Field, ItemMod, Item, ItemStruct, parse_str};
use util::{field_helpers::extract_fields, mod_helpers::{get_type_name, has_warcmutex_attribute}};

use crate::util::impl_block_helpers::{
    change_block_method, get_impl_type_name, remove_pub_from_impl_item,
    transform_method_return_type,
};

mod util;

#[proc_macro_attribute]
pub fn warcmutex(_ : TokenStream, input : TokenStream) -> TokenStream{
    //obtem do o módulo
    let item: Item = parse_macro_input!(input as Item);
    match item {
        Item::Impl(item_impl) => extend_impl(item_impl).into(),
        Item::Mod(sub_mod) => extend_mod(sub_mod).into(),
        Item::Struct(item_struct ) => extend_struct(item_struct).into(),
        _ => panic!("This macro can only be used in structs, impl Blocks and mods!"),
    }
}

fn extend_mod(item_mod : ItemMod) -> TokenStream{
    let mut tokens = format!("{} mod {}",item_mod.vis.into_token_stream().to_string(),item_mod.ident.to_string());
    tokens += "{";
    for item in item_mod.content.unwrap().1.iter(){
        if !has_warcmutex_attribute(&item){
            match item.clone() {
                Item::Impl(item_impl) => tokens += &extend_impl(item_impl.clone()).to_string(),
                Item::Mod(sub_mod) => tokens += &extend_mod(sub_mod.clone()).to_string(),
                Item::Struct(item_struct ) => tokens += &extend_struct(item_struct.clone()).to_string(),
                _ => tokens += &item.into_token_stream().to_string(),
            }
        }
    }
    tokens += "}";
    let output : proc_macro2::TokenStream = parse_str(&tokens).unwrap();
    //println!(">>Mod>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n {} <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<",output);
    output.into()
}

fn extend_struct(item_struct : ItemStruct) -> TokenStream{
    
    let input: DeriveInput = parse_quote! {
        #item_struct
    };
    let vis = &input.vis;
    let field_names = extract_fields(&input);
    // Gere um TokenStream contendo apenas os nomes dos campos impl Iterator<Item = TokenStream>
    let field_tokens = field_names.iter().map(|field_name| {
        let new_field_name = if let Visibility::Public(_) = field_name.vis{
            Field{
                attrs: field_name.attrs.clone(),
                vis: Visibility::Inherited,
                mutability: field_name.mutability.clone(),
                ident: field_name.ident.clone(),
                colon_token: field_name.colon_token,
                ty: field_name.ty.clone(),
            }
        }else{
            field_name.clone()
        };
        quote! { #new_field_name }
    });
    // Clone the struct name identifier and assign a new name
    let original_struct_name = input.ident.clone();
    let base_name = original_struct_name.to_string()+"Base";
    let base_name : Ident = parse_str(&base_name).unwrap();

    // Generate the transformed struct definition
    let transformed_struct = quote! {
        pub struct #base_name {
            // Fields remain intact
            #(#field_tokens),*
        }
    };
    // Generate the second struct definition
    let secund_struct = quote! {
        #vis struct #original_struct_name {
            // New field named 'base' of type 'Arc<Base>'
            base: std::sync::Arc<std::sync::Mutex<#base_name>>,
        }
    };

    let lock_impl: ItemImpl = parse_quote!(
        impl #original_struct_name{
            pub fn lock(&mut self) -> LockResult<MutexGuard<'_, #base_name>> {
                self.base.lock()
            }
        }
    );

    let clone_impl: ItemImpl = parse_quote!(
        impl Clone for #original_struct_name{
            fn clone(&self) -> Self {
                Self { base: self.base.clone() }
            }
        }
    );

    let send_impl: ItemImpl = parse_quote!(        
        unsafe impl Send for #original_struct_name{}
    );

    let sync_impl: ItemImpl = parse_quote!(        
        unsafe impl Sync for #original_struct_name{}
    );

    // Combine the transformed struct and the original struct into the output tokens
    let output = quote! {
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::sync::LockResult;
        use std::sync::MutexGuard;
        #transformed_struct
        #secund_struct
        #lock_impl
        #clone_impl
        #send_impl
        #sync_impl
    };
    // Return the generated code as a TokenStream
    //println!(">>Struct>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n {} <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<",output);
    output.into()
}

fn extend_impl(item_impl : ItemImpl) -> TokenStream{
    let original_name = item_impl.self_ty.clone();
    let original_name_str = get_type_name( original_name.clone()).expect("Could not get type name.");
    let base_name_str = original_name_str+"Base";
    let base_name : Ident = parse_str(&base_name_str).unwrap();

    // Obter todos os itens de implementação
    let mut original_impl_items = item_impl.items.clone();
    original_impl_items
        .iter_mut()
        .for_each(|ii| transform_method_return_type(ii, get_impl_type_name(&item_impl).unwrap()));
    let first_self_ty = parse_quote!(#base_name);
    let sencund_self_ty = parse_quote!(#original_name);

    let mut first_impl_items: Vec<ImplItem> = vec![];
    let mut secund_impl_items: Vec<ImplItem> = vec![];
    for original_impl_item in original_impl_items {
        if let Some(item) = change_block_method(&original_impl_item,base_name_str.clone()) {
            secund_impl_items.push(item);
            let mut modificated_impl_item = original_impl_item.clone();
            remove_pub_from_impl_item(&mut modificated_impl_item);
            first_impl_items.push(modificated_impl_item);
        } else {
            first_impl_items.push(original_impl_item.clone());
        }
    }

    let secund_impl_block = ItemImpl {
        attrs: item_impl.attrs.clone(),
        defaultness: item_impl.defaultness,
        unsafety: item_impl.unsafety,
        impl_token: item_impl.impl_token,
        generics: item_impl.generics.clone(),
        trait_: item_impl.trait_.clone(),
        self_ty: sencund_self_ty,
        brace_token: item_impl.brace_token,
        items: secund_impl_items,
    };
    // Crie um novo bloco impl com o nome modificado do tipo
    let renamed_impl_block = ItemImpl {
        attrs: item_impl.attrs,
        defaultness: item_impl.defaultness,
        unsafety: item_impl.unsafety,
        impl_token: item_impl.impl_token,
        generics: item_impl.generics,
        trait_: item_impl.trait_,
        self_ty: first_self_ty,
        brace_token: item_impl.brace_token,
        items: first_impl_items,
    };
    // Retorne o código modificado como TokenStream
    let output = quote! {
        #renamed_impl_block
        #secund_impl_block
    };
    //println!(">>Impl>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n {} <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<",output);
    output.into()
}

/*#[proc_macro_attribute]
pub fn warcmutex_struct(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let vis = &input.vis;
    let field_names = extract_fields(&input);
    // Gere um TokenStream contendo apenas os nomes dos campos impl Iterator<Item = TokenStream>
    let field_tokens = field_names.iter().map(|field_name| {
        let new_field_name = if let Visibility::Public(_) = field_name.vis{
            Field{
                attrs: field_name.attrs.clone(),
                vis: Visibility::Inherited,
                mutability: field_name.mutability.clone(),
                ident: field_name.ident.clone(),
                colon_token: field_name.colon_token,
                ty: field_name.ty.clone(),
            }
        }else{
            field_name.clone()
        };
        quote! { #new_field_name }
    });
    // Clone the struct name identifier and assign a new name
    let original_struct_name = input.ident.clone();

    // Generate the transformed struct definition
    let transformed_struct = quote! {
        pub struct Base {
            // Fields remain intact
            #(#field_tokens),*
        }
    };
    // Generate the second struct definition
    let secund_struct = quote! {
        #vis struct #original_struct_name {
            // New field named 'base' of type 'Arc<Base>'
            base: std::sync::Arc<std::sync::Mutex<Base>>,
        }
    };

    let lock_impl: ItemImpl = parse_quote!(
        impl #original_struct_name{
            pub fn lock(&mut self) -> LockResult<MutexGuard<'_, Base>> {
                self.base.lock()
            }
        }
    );

    // Combine the transformed struct and the original struct into the output tokens
    let output = quote! {
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::sync::LockResult;
        use std::sync::MutexGuard;
        #transformed_struct
        #secund_struct
        #lock_impl
    };
    // Return the generated code as a TokenStream
    output.into()
}

#[proc_macro_attribute]
pub fn warcmutex_impl(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse o nome original do bloco impl
    // Parse o input como um ItemImpl
    let impl_block = parse_macro_input!(input as ItemImpl);
    let original_name = impl_block.self_ty.clone();

    // Obter todos os itens de implementação
    let mut original_impl_items = impl_block.items.clone();
    original_impl_items
        .iter_mut()
        .for_each(|ii| transform_method_return_type(ii, get_impl_type_name(&impl_block).unwrap()));
    let first_self_ty = parse_quote!(Base);
    let sencund_self_ty = parse_quote!(#original_name);

    let mut first_impl_items: Vec<ImplItem> = vec![];
    let mut secund_impl_items: Vec<ImplItem> = vec![];
    for original_impl_item in original_impl_items {
        if let Some(item) = change_block_method(&original_impl_item) {
            secund_impl_items.push(item);
            let mut modificated_impl_item = original_impl_item.clone();
            remove_pub_from_impl_item(&mut modificated_impl_item);
            first_impl_items.push(modificated_impl_item);
        } else {
            first_impl_items.push(original_impl_item.clone());
        }
    }

    let secund_impl_block = ItemImpl {
        attrs: impl_block.attrs.clone(),
        defaultness: impl_block.defaultness,
        unsafety: impl_block.unsafety,
        impl_token: impl_block.impl_token,
        generics: impl_block.generics.clone(),
        trait_: impl_block.trait_.clone(),
        self_ty: sencund_self_ty,
        brace_token: impl_block.brace_token,
        items: secund_impl_items,
    };
    // Crie um novo bloco impl com o nome modificado do tipo
    let renamed_impl_block = ItemImpl {
        attrs: impl_block.attrs,
        defaultness: impl_block.defaultness,
        unsafety: impl_block.unsafety,
        impl_token: impl_block.impl_token,
        generics: impl_block.generics,
        trait_: impl_block.trait_,
        self_ty: first_self_ty,
        brace_token: impl_block.brace_token,
        items: first_impl_items,
    };
    // Retorne o código modificado como TokenStream
    let output = quote! {
        #renamed_impl_block
        #secund_impl_block
    };

    /*println!(
        ">>>>>>>>>>>>>> resultado impl >>>>>>>>>>>>>>> \n{}\n <<<<<<<<<<<<<<<<<<<<>>>>>>>>>>>>>>>>>>>>",
        output.to_string()
    );*/

    output.into()
}
*/