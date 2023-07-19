use syn::{Attribute, Item, Type};

pub fn has_warcmutex_attribute(item : &Item) -> bool{
    if let Item::Struct(struct_) = item{
        return struct_.attrs.iter().find(|&attr| {
            let name = get_name_attribute(attr).unwrap();
            name.contains("warcmutex")
        }).is_some()
    }else if let Item::Mod(mod_) = item{
        return mod_.attrs.iter().find(|&attr| {
            let name = get_name_attribute(attr).unwrap();
            name.contains("warcmutex")
        }).is_some()
    }
    else if let Item::Impl(impl_) = item{
        return impl_.attrs.iter().find(|&attr| {
            let name = get_name_attribute(attr).unwrap();
            name.contains("warcmutex")
        }).is_some()
    }
    false
}




fn get_name_attribute(attribute: &Attribute) -> Option<String> {
    if let Some(path) = attribute.path().get_ident() {
        let attribute_name = path.to_string();
        return Some(attribute_name);
    }
    None
}

pub fn get_type_name(type_ : Box<Type>) -> Option<String>{
    if let Type::Path(path) = type_.as_ref(){
        if let Some(ident) = path.path.get_ident(){
            return Some(ident.to_string())
        }
    }
    None
}
