use std::collections::HashSet;

use unsynn::{
    BraceGroup, BraceGroupContaining, Colon, Comma, DelimitedVec, Ident, Literal, Or,
    ParenthesisGroupContaining, Parse, ToTokens, TokenStreamUntil, TrailingDelimiter, format_ident,
};

unsynn::unsynn! {
    keyword Valkey = "valkey";
    keyword Prefix = "prefix";
    keyword Key = "key";
    keyword Ttl = "ttl";
    keyword Refresh = "refresh";
    keyword Map = "map";
    keyword Args = "args";

    struct Closure {
        _left: Or,
        _right: Or,

        body: TokenStreamUntil<Comma>
    }

    struct PopulatedClosure {
        _left: Or,
        value: Ident,
        _right: Or,

        body: TokenStreamUntil<Comma>

    }

    struct MapArg {
        map: Map,
        _colon: Colon,

        closure: PopulatedClosure,
    }

    struct CacheArg {
        name: Ident,
        _colon: Colon,
        ty: Ident,
    }

    struct KeyArg {
        _key_kw: Key,
        _key_colon: Colon,
        key: TokenStreamUntil<Comma>,
        _key_comma: Comma,
    }

    struct Arg {
        name: Ident,
        _colon: Colon,
        ty: Ident,
    }

    type ArgList = DelimitedVec<
        Arg,
        Comma,
        TrailingDelimiter::Optional,
        1
    >;

    struct ArgsArg {
        _args_kw: Args,
        _args_colon: Colon,
        args: ParenthesisGroupContaining<ArgList>,
        _args_comma: Comma,
    }

    struct TtlArg {
        _ttl_kw: Ttl,
        _ttl_colon: Colon,
        ttl: TokenStreamUntil<Comma>,
        _ttl_comma: Comma,
    }

    struct RefreshArg {
        _refresh_kw: Refresh,
        _refresh_colon: Colon,
        closure: Closure,
        _refresh_comma: Comma,
    }

    struct CacheOptions {
        key: KeyArg,
        args: Option<ArgsArg>,
        ttl: TtlArg,
        refresh: RefreshArg,
        map: Option<MapArg>,

        _trailing_comma: Option<Comma>,
    }

    struct CacheEntry {
        name: Ident,
        _colon: Colon,
        value_type: TokenStreamUntil<BraceGroup>,

        options: BraceGroupContaining<CacheOptions>,
    }

    type CacheEntries = DelimitedVec<
        CacheEntry,
        Comma,
        TrailingDelimiter::Optional,
        1
    >;

    struct CacheInput {
        implementor: Ident,
        _comma: Comma,

        entries: CacheEntries,
    }
}

#[proc_macro]
pub fn valkey_cache(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(message) => {
            let message = Literal::string(&message).to_string();

            quote::quote! {
                compile_error!(#message)
            }
            .into()
        }
    }
}

fn expand(input: unsynn::TokenStream) -> Result<unsynn::TokenStream, String> {
    let mut tokens = input.into_token_iter();

    let input = CacheInput::parse_all(&mut tokens)
        .map_err(|err| format!("Invalid cache entry declaration: {err:?}"))?;

    let implementor = input.implementor;
    let mut methods: Vec<unsynn::TokenStream> = Vec::new();
    let mut names = HashSet::new();
    let mut keys = HashSet::new();

    for delimited in input.entries.iter() {
        let entry = &delimited.value;

        let name = entry.name.clone();

        if !names.insert(name.clone()) {
            return Err(format!("duplicate cache entry `{name}`"));
        }

        let options = &entry.options.content;

        let key = options.key.key.to_token_stream();
        if !keys.insert(key.to_string()) {
            return Err(format!("duplicate key for `{name}`"));
        }

        let ttl = options.ttl.ttl.to_token_stream();

        let value_type = entry.value_type.to_token_stream();
        let refresh = options.refresh.closure.to_token_stream();

        let select = select(&options.map);

        let mut arg_names = Vec::new();
        let mut arg_types = Vec::new();

        if let Some(args) = &options.args {
            for arg in args.args.content.iter() {
                arg_names.push(arg.value.name.clone());
                arg_types.push(arg.value.ty.clone());
            }
        }

        let get_method = format_ident!("get_{}", name);
        let set_method = format_ident!("set_{}", name);

        methods.push(
            quote::quote! {
                pub async fn #get_method(&self #(, #arg_names: #arg_types)*) -> redis::RedisResult<#value_type> {
                    let key = #key;

                    let value = ::mnemosyne_cache::get_or_refresh(
                        &self.cache,
                        &key,
                        ::std::time::Duration::from_secs(#ttl),
                        #refresh.map_err(|e| ::mnemosyne_cache::utils::codec_error("Failed to refresh", e)),
                        #select,
                    ).await;

                    value
                }

                pub async fn #set_method(&self #(, #arg_names: #arg_types)*, value: &#value_type) -> redis::RedisResult<()> {
                    let key = #key;

                    ::mnemosyne_cache::set(
                        &self.cache,
                        &key,
                        value,
                        ::std::time::Duration::from_secs(#ttl),
                    ).await
                }
            }
            .into(),
        )
    }

    Ok(quote::quote! {
        impl #implementor {
            #(#methods)*
        }
    }
    .into())
}

fn select(method: &Option<MapArg>) -> unsynn::TokenStream {
    match method {
        Some(m) => {
            let func: unsynn::TokenStream = m.closure.to_token_iter().collect();
            quote::quote! {
                #func
            }
        }
        None => {
            quote::quote! {
                |v| v.into()
            }
        }
    }
}
