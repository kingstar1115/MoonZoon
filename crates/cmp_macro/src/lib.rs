use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, ItemFn, ReturnType, Type};
use uuid::Uuid;

/* 
#[s_var]
fn counter_count() -> i32 {
    3
}

// or 

#[s_var]
fn counter_count() -> SVar<i32> {
    3
}

// to 

fn counter_count() -> SVar<i32> {
    const __ID: u128 = 0x936DA01F9ABD4D9D80C702AF85C822A8;
    const __BLOCK: __Block = __Block::SVar(__ID);
    __Relations::add_dependency(__BLOCK);
    s_var(__ID, || {
        __BlockCallStack::push(__BLOCK);
        let output = (|| {
            3
        })();
        __BlockCallStack::pop();
        output
    })
}
*/
#[proc_macro_attribute]
pub fn cmp(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
    // let mut input_fn: ItemFn = syn::parse(input).unwrap();

    // let id = Uuid::new_v4().as_u128();

    // let inner_block = input_fn.block;
    // input_fn.block = parse_quote!({ 
    //     const __ID: u128 = #id;
    //     const __BLOCK: __Block = __Block::SVar(__ID);
    //     __Relations::add_dependency(__BLOCK);
    //     s_var(__ID, || {
    //         __BlockCallStack::push(__BLOCK);
    //         let output = (|| #inner_block)();
    //         __BlockCallStack::pop();
    //         output
    //     })
    // });

    // input_fn.sig.output = match input_fn.sig.output {
    //     ReturnType::Default => {
    //         parse_quote!(-> SVar<()>)
    //     },
    //     ReturnType::Type(_, type_) if is_first_segment_s_var(&type_)  => {
    //         parse_quote!(-> #type_)
    //     },
    //     ReturnType::Type(_, type_) => {
    //         parse_quote!(-> SVar<#type_>)
    //     },
    // };

    // input_fn.into_token_stream().into()
}

fn is_first_segment_s_var(type_: &Box<Type>) -> bool {
    if let Type::Path(type_path) = type_.as_ref() {
        type_path
            .path
            .segments
            .first()
            .map(|path_segment| path_segment.ident == "SVar")
            .unwrap_or_default()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
