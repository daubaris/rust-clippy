use crate::consts::{constant_simple, Constant};
use rustc::hir::*;
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
use rustc::{declare_lint, lint_array};
use syntax::source_map::Span;
use crate::utils::{in_macro, snippet, span_lint, unsext, clip};
use rustc::ty;

/// **What it does:** Checks for identity operations, e.g. `x + 0`.
///
/// **Why is this bad?** This code can be removed without changing the
/// meaning. So it just obscures what's going on. Delete it mercilessly.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// x / 1 + 0 * 1 - 0 | 0
/// ```
declare_clippy_lint! {
    pub IDENTITY_OP,
    complexity,
    "using identity operations, e.g. `x + 0` or `y / 1`"
}

#[derive(Copy, Clone)]
pub struct IdentityOp;

impl LintPass for IdentityOp {
    fn get_lints(&self) -> LintArray {
        lint_array!(IDENTITY_OP)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for IdentityOp {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, e: &'tcx Expr) {
        if in_macro(e.span) {
            return;
        }
        if let ExprKind::Binary(ref cmp, ref left, ref right) = e.node {
            match cmp.node {
                BinOpKind::Add | BinOpKind::BitOr | BinOpKind::BitXor => {
                    check(cx, left, 0, e.span, right.span);
                    check(cx, right, 0, e.span, left.span);
                },
                BinOpKind::Shl | BinOpKind::Shr | BinOpKind::Sub => check(cx, right, 0, e.span, left.span),
                BinOpKind::Mul => {
                    check(cx, left, 1, e.span, right.span);
                    check(cx, right, 1, e.span, left.span);
                },
                BinOpKind::Div => check(cx, right, 1, e.span, left.span),
                BinOpKind::BitAnd => {
                    check(cx, left, -1, e.span, right.span);
                    check(cx, right, -1, e.span, left.span);
                },
                _ => (),
            }
        }
    }
}

#[allow(cast_possible_wrap)]
fn check(cx: &LateContext<'_, '_>, e: &Expr, m: i8, span: Span, arg: Span) {
    if let Some(Constant::Int(v)) = constant_simple(cx, cx.tables, e) {
        let check = match cx.tables.expr_ty(e).sty {
            ty::Int(ity) => unsext(cx.tcx, -1_i128, ity),
            ty::Uint(uty) => clip(cx.tcx, !0, uty),
            _ => return,
        };
        if match m {
            0 => v == 0,
            -1 => v == check,
            1 => v == 1,
            _ => unreachable!(),
        } {
            span_lint(
                cx,
                IDENTITY_OP,
                span,
                &format!(
                    "the operation is ineffective. Consider reducing it to `{}`",
                    snippet(cx, arg, "..")
                ),
            );
        }
    }
}
