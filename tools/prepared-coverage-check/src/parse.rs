//! Strict parsers for the adapter macro inputs admitted as evidence.

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, LitStr, Path, Token, Type};

pub(crate) struct EndpointWireArgs {
    pub(crate) ty: Type,
    pub(crate) shape: Expr,
    pub(crate) response: Expr,
    pub(crate) mapping: Expr,
    pub(crate) destructive: Expr,
    pub(crate) cost: Expr,
}

impl Parse for EndpointWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
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
            ty,
            shape,
            response,
            mapping,
            destructive,
            cost,
        })
    }
}

pub(crate) struct BodyWireArgs {
    pub(crate) ty: Type,
    pub(crate) endpoint: Expr,
    pub(crate) key: LitStr,
    pub(crate) writer: Path,
}

impl Parse for BodyWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let endpoint: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let writer: Path = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected body_wire tokens"));
        }
        Ok(Self {
            ty,
            endpoint,
            key,
            writer,
        })
    }
}

pub(crate) struct QueryWireArgs {
    pub(crate) ty: Type,
    pub(crate) endpoint: Expr,
}

impl Parse for QueryWireArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let endpoint: Expr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected query_wire tokens"));
        }
        Ok(Self { ty, endpoint })
    }
}

pub(crate) struct BodyComponentArgs {
    pub(crate) ty: Type,
    pub(crate) key: LitStr,
    pub(crate) writer: Path,
}

impl Parse for BodyComponentArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let writer: Path = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected body_component tokens"));
        }
        Ok(Self { ty, key, writer })
    }
}

pub(crate) struct EndpointPrepareArgs {
    pub(crate) types: Punctuated<Type, Token![,]>,
}

impl Parse for EndpointPrepareArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let types = Punctuated::parse_terminated(input)?;
        if types.is_empty() {
            return Err(input.error("impl_endpoint_prepare requires at least one type"));
        }
        Ok(Self { types })
    }
}
