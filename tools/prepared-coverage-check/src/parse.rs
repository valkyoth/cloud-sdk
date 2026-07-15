//! Strict parsers for the adapter macro inputs admitted as evidence.

use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Path, Token, Type};

pub(crate) struct EndpointWireArgs {
    pub(crate) shape: Expr,
    pub(crate) response: Expr,
    pub(crate) mapping: Expr,
    pub(crate) destructive: Expr,
    pub(crate) cost: Expr,
}

impl Parse for EndpointWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let shape: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let response: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let mapping: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let destructive: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let cost: Expr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected endpoint_wire tokens"));
        }
        Ok(Self {
            shape,
            response,
            mapping,
            destructive,
            cost,
        })
    }
}

pub(crate) struct BodyWireArgs {
    pub(crate) endpoint: Expr,
    pub(crate) key: LitStr,
}

impl Parse for BodyWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let endpoint: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Path = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected body_wire tokens"));
        }
        Ok(Self { endpoint, key })
    }
}

pub(crate) struct QueryWireArgs {
    pub(crate) endpoint: Expr,
}

impl Parse for QueryWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let endpoint: Expr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected query_wire tokens"));
        }
        Ok(Self { endpoint })
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
        let scrutinee: Expr = input.parse()?;
        let canonical_scrutinee = matches!(
            &scrutinee,
            Expr::Path(path)
                if path.attrs.is_empty()
                    && path.qself.is_none()
                    && path.path.leading_colon.is_none()
                    && path.path.is_ident("operation_key")
        );
        if !canonical_scrutinee {
            return Err(input.error("accepts_operation must match the operation_key parameter"));
        }
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
