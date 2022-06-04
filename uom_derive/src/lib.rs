trait UomEquivalence {
    type Equivalent;
    fn to_equivalent(&self) -> Self::Equivalent;
}

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(UomEquivalence)]
pub fn derive_uom_equivalence_fn(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
