use proc_macro::TokenStream;
use syn::{parse_quote, spanned::Spanned, ItemFn};

#[proc_macro_attribute]
pub fn tracked_call(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn: ItemFn = syn::parse(input).unwrap();

    let inner_block = input_fn.block;
    input_fn.block = parse_quote!({ 
        let __tracked_call = __TrackedCall::create();
        // log!("from macro: {:#?}", __tracked_call_id);
        __TrackedCallStack::push(std::rc::Rc::new(std::cell::RefCell::new(__tracked_call))); 
        let output = (move || #inner_block)();
        __TrackedCallStack::pop();
        output
    });

    quote::quote_spanned!(input_fn.span()=>
        #input_fn
    ).into()
}
