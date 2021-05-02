use crate::{make_flags,  Element, IntoElement, IntoOptionElement};
use dominator::{Dom, DomBuilder};
use futures_signals::{signal::{Signal, SignalExt}, signal_vec::{SignalVec, SignalVecExt}};
use std::marker::PhantomData;

// ------ ------
//   Element 
// ------ ------

make_flags!(Empty);

pub struct Row<EmptyFlag> {
    dom_builder:DomBuilder<web_sys::HtmlElement>,
    flags: PhantomData<EmptyFlag>
}

impl Row<EmptyFlagSet> {
    pub fn new() -> Self {
        Self {
            dom_builder: DomBuilder::new_html("div").class("row"),
            flags: PhantomData,
        }
    }
}

impl Element for Row<EmptyFlagNotSet> {
    fn render(self) -> Dom {
        self.dom_builder.into_dom()
    }
}

// ------ ------
//  Attributes 
// ------ ------

impl<'a, EmptyFlag> Row<EmptyFlag> {
    pub fn item(self, 
        item: impl IntoOptionElement<'a> + 'a
    ) -> Row<EmptyFlagNotSet> {
        let dom_builder = if let Some(item) = item.into_option_element() {
            self.dom_builder.child(item.render())
        } else {
            self.dom_builder
        };
        Row {
            dom_builder,
            flags: PhantomData
        }
    }

    pub fn item_signal(
        self, 
        item: impl Signal<Item = impl IntoOptionElement<'a>> + Unpin + 'static
    ) -> Row<EmptyFlagNotSet> {
        Row {
            dom_builder: self.dom_builder.child_signal(
                item.map(|item| item.into_option_element().map(|element| element.render()))
            ),
            flags: PhantomData
        }
    }

    pub fn items(self, 
        items: impl IntoIterator<Item = impl IntoElement<'a> + 'a>
    ) -> Row<EmptyFlagNotSet> {
        Row {
            dom_builder: self.dom_builder.children(
                items.into_iter().map(|item| item.into_element().render())
            ),
            flags: PhantomData
        }
    }

    pub fn items_signal_vec(
        self, 
        items: impl SignalVec<Item = impl IntoElement<'a>> + Unpin + 'static
    ) -> Row<EmptyFlagNotSet> {
        Row {
            dom_builder: self.dom_builder.children_signal_vec(
                items.map(|item| item.into_element().render())
            ),
            flags: PhantomData
        }
    }
} 
