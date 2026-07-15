//! Strict parsers for the adapter macro inputs admitted as evidence.

use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Path, Token, Type};

pub(crate) struct EndpointWireArgs {
    pub(crate) mapping: Expr,
}

impl Parse for EndpointWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let _: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let mapping: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Expr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected endpoint_wire tokens"));
        }
        Ok(Self { mapping })
    }
}

pub(crate) struct BodyWireArgs {
    pub(crate) key: LitStr,
}

impl Parse for BodyWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let _: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Path = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected body_wire tokens"));
        }
        Ok(Self { key })
    }
}

pub(crate) struct BodyComponentArgs {
    pub(crate) key: LitStr,
}

impl Parse for BodyComponentArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Path = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected body_component tokens"));
        }
        Ok(Self { key })
    }
}

pub(crate) struct AcceptedKeys {
    pub(crate) keys: Vec<LitStr>,
}

impl Parse for AcceptedKeys {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let mut keys = Vec::new();
        keys.push(input.parse()?);
        while input.peek(Token![|]) {
            input.parse::<Token![|]>()?;
            keys.push(input.parse()?);
        }
        if !input.is_empty() {
            return Err(input.error("accepts_operation must match only string literals"));
        }
        Ok(Self { keys })
    }
}
