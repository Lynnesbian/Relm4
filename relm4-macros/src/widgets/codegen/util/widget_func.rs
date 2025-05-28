use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::{Error, token};
use syn::{Ident, spanned::Spanned};

use crate::widgets::{Widget, WidgetFunc, WidgetTemplateAttr};

impl Widget {
    /// Get tokens for the widget's type.
    pub(crate) fn func_type_token_stream(&self) -> TokenStream2 {
        let is_local = self.attr.is_local_attr();
        let func = &self.func;
        let path = &self.func.path;
        let mut tokens = TokenStream2::new();

        // If type was specified, use it
        let (type_segments, num_of_segments) = if let Some(ty) = &func.ty {
            return ty.to_token_stream();
        } else if is_local {
            return Error::new(func.span().unwrap().into(),
                    format!("You need to specify the type of the local variable. Use this instead: {} -> Type {{ ...", 
                    self.name)).into_compile_error();
        } else if func.args.is_some() {
            // If for example gtk::Box::new() was used, ignore ::new()
            // and use gtk::Box as type.
            let len = path.segments.len();
            if len == 0 {
                unreachable!("Path can't be empty");
            } else if self.template_attr == WidgetTemplateAttr::Template {
                (&path.segments, path.segments.len())
            } else if len == 1 {
                return Error::new(func.span().unwrap().into(),
                        format!("You need to specify a type of your function. Use this instead: {}() -> Type {{ ...",
                        path.to_token_stream())).into_compile_error();
            } else {
                (&path.segments, len - 1)
            }
        } else {
            (&path.segments, path.segments.len())
        };

        let mut seg_iter = type_segments.iter().take(num_of_segments);
        let first = if let Some(first) = seg_iter.next() {
            first
        } else {
            return Error::new(
                func.span().unwrap().into(),
                "No path segments in WidgetFunc.",
            )
            .into_compile_error();
        };
        tokens.extend(first.to_token_stream());

        for segment in seg_iter {
            tokens.extend(quote! {::});
            tokens.extend(segment.to_token_stream());
        }

        tokens
    }
}

impl WidgetFunc {
    /// Get the tokens of the widget's function.
    pub(crate) fn func_token_stream(&self) -> TokenStream2 {
        let WidgetFunc {
            path,
            args,
            method_chain,
            ..
        } = &self;

        let mut stream = if let Some(args) = args {
            quote! { #path(#args) }
        } else if method_chain.is_some() {
            path.to_token_stream()
        } else {
            quote_spanned! {
                path.span() => #path::default()
            }
        };

        if let Some(method_chain) = method_chain {
            stream.extend(quote! {
                .#method_chain
            });
        }

        stream
    }

    pub(crate) fn widget_template_path(
        &self,
        template_widget_name: &Ident,
        widget_name: &Ident,
    ) -> Punctuated<Ident, token::Dot> {
        let mut template_path = Punctuated::new();
        template_path.push(template_widget_name.clone());
        template_path.push(widget_name.clone());
        if let Some(chain) = &self.method_chain {
            for method in chain {
                if method.turbofish.is_some() || method.args.is_some() {
                    break;
                } else {
                    template_path.push(method.ident.clone());
                }
            }
            template_path
        } else {
            template_path
        }
    }

    pub(crate) fn widget_template_init(&self) -> TokenStream2 {
        let widget_ty = &self.path;
        let args = if let Some(args) = &self.args {
            args.into_token_stream()
        } else {
            quote_spanned! { self.path.span() => () }
        };
        quote! {
            <#widget_ty as relm4::WidgetTemplate>::init(#args)
        }
    }
}
