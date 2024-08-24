use proc_macro::TokenStream;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{Expr, ItemFn, ReturnType, Stmt, StmtMacro};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn panic_to_result(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast: ItemFn = syn::parse(input).expect("expected function");

    ast.sig.output = wrap_output_type_with_result(&ast).expect("failed to wrap output with result");
    wrap_output_with_result(&mut ast).expect("failed to wrap output with result");

    ast.block.stmts = match ast
        .block
        .stmts
        .into_iter()
        .map(convert_panic_to_result)
        .collect::<Result<_, _>>()
    {
        Ok(stmts) => stmts,
        Err(err) => return err.to_compile_error().into(),
    };

    quote!(
        #ast
    )
    .into()
}

/// Returns the ReturnType of the function wrapped in a [`Result`].
fn wrap_output_type_with_result(func: &ItemFn) -> syn::Result<ReturnType> {
    let result_output = match &func.sig.output {
        ReturnType::Default => quote!(-> Result<(), std::boxed::Box<dyn std::error::Error>>),
        ReturnType::Type(_, ty) => {
            if ty.into_token_stream().to_string().contains("Result") {
                emit_error!(
                    ty, "cannot apply macro to functions which return Result";
                    help = "remove the Result from the return type"
                );
            }
            quote!(-> Result<#ty, std::boxed::Box<dyn std::error::Error>>)
        }
    };
    syn::parse2(result_output)
}

/// Wraps the output of `func` with a [`Result::Ok`]
fn wrap_output_with_result(func: &mut ItemFn) -> syn::Result<()> {
    let Some(last_stmt) = func.block.stmts.pop() else {
        emit_error!(
            func.sig.ident,
            "cannot apply macro to empty function";
            help = "add a return statement to the function"
        );
        return Ok(());
    };
    let result_wrapped_last_stmt = quote!(Ok(#last_stmt));
    let parsed_last_stmt = Stmt::Expr(syn::parse2(result_wrapped_last_stmt)?, None);
    func.block.stmts.push(parsed_last_stmt);

    Ok(())
}

/// Returns an [`Result`] instead of all panic for an if expression containing
/// a panic! macro.
fn convert_panic_to_result(stmt: Stmt) -> syn::Result<Stmt> {
    match stmt {
        Stmt::Expr(expr, token) => match expr {
            Expr::If(mut if_expr) => {
                let new_stmts: Vec<Stmt> = if_expr
                    .then_branch
                    .stmts
                    .into_iter()
                    .map(|stmt| match stmt {
                        Stmt::Macro(ref m) => {
                            if is_panic_macro(m) {
                                let content = get_macro_content(m);
                                if content.is_empty() {
                                    emit_error!(
                                        m,
                                        "cannot convert panic macro without message";
                                        help = "add a message to the panic macro"
                                    );
                                }
                                return syn::parse2(quote!(return Err(#content.into());));
                            }
                            Ok(stmt)
                        }
                        _ => Ok(stmt),
                    })
                    .collect::<Result<_, _>>()?;

                if_expr.then_branch.stmts = new_stmts;
                Ok(Stmt::Expr(Expr::If(if_expr), token))
            }
            _ => Ok(Stmt::Expr(expr, token)),
        },
        _ => Ok(stmt),
    }
}

/// Returns true if the macro is a `panic!` macro
fn is_panic_macro(m: &StmtMacro) -> bool {
    m.mac
        .path
        .segments
        .iter()
        .any(|s| s.ident.to_string().contains("panic"))
}

/// Returns the content of the provided macro statement.
fn get_macro_content(m: &StmtMacro) -> &proc_macro2::TokenStream {
    &m.mac.tokens
}
