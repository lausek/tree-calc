pub mod context;

use self::context::*;
use crate::ast::*;

pub type VmValue = Value;
pub type VmError = String;
pub type VmResult = Result<VmValue, VmError>;

pub struct Vm
{
    ctx: Box<dyn Lookable>,
}

impl Vm
{
    pub fn new() -> Self
    {
        Self {
            ctx: Box::new(VmContext::new()),
        }
    }

    pub fn with_stdlib() -> Self
    {
        Self {
            ctx: Box::new(VmContext::stdlib()),
        }
    }

    pub fn run(&mut self, expr: &Expr) -> VmResult
    {
        run_with_ctx(expr, &mut self.ctx)
    }
}

pub fn run_with_ctx(expr: &Expr, ctx: &mut Box<dyn Lookable>) -> VmResult
{
    match expr {
        Expr::Value(v) => Ok(v.clone()),
        Expr::Ref(name) => run_lookup(name, ctx),
        Expr::Func(name, params) => run_function(name, params, ctx),
        Expr::Comp(Operator::Equ, lhs, rhs) => match lhs {
            box Expr::Func(name, params) => {
                let func = VmFunction::Virtual((params.clone(), rhs.clone()));
                ctx.set(name, func);
                Ok(Value::Empty)
            }
            box Expr::Ref(name) => {
                let val = VmFunction::Virtual((vec![], rhs.clone()));
                ctx.set(name, val);
                Ok(Value::Empty)
            }
            _ => Err(format!("`{:?}` is not assignable", lhs)),
        },
        Expr::Comp(op, arg1, arg2) => {
            let arg1 = run_with_ctx(arg1, ctx)?;
            let arg2 = run_with_ctx(arg2, ctx)?;

            let arg1 = NumType::from(arg1);
            let arg2 = NumType::from(arg2);

            let result = match op {
                Operator::Add => arg1 + arg2,
                Operator::Sub => arg1 - arg2,
                Operator::Mul => arg1 * arg2,
                Operator::Div => {
                    if arg2 != 0.0 {
                        arg1 / arg2
                    } else {
                        return Err("division with 0".to_string());
                    }
                }
                Operator::Pow => arg1.powf(arg2),
                Operator::Mod => arg1 % arg2,
                _ => unreachable!(),
            };

            Ok(result.into())
        }
    }
}

pub fn run_lookup(name: &RefType, ctx: &mut Box<dyn Lookable>) -> VmResult
{
    if let Some(entry) = ctx.get(name) {
        match &*(entry.borrow()) {
            // TODO: find a way around that clone
            VmFunction::Virtual((_args, n)) => run_with_ctx(&n.clone(), ctx),
            // TODO: there must be an easier way to specify empty params. Option<Vec<>> maybe?
            VmFunction::Native(func) => func(&vec![], ctx),
        }
    } else {
        Err(format!("variable `{}` is unknown", name))
    }
}

pub fn run_function(name: &RefType, params: &TupleType, ctx: &mut Box<dyn Lookable>) -> VmResult
{
    let params = run_tuple_exprs(params, ctx)?;
    if let Some(entry) = ctx.get(name) {
        // TODO: execute all params before pushing them into a function
        match &*(entry.borrow()) {
            // TODO: find a way around that clone
            VmFunction::Virtual((args, expr)) => {
                if args.len() != params.len() {
                    return Err(format!(
                        "function `{}` expected {} argument, got {}",
                        name,
                        args.len(),
                        params.len()
                    ));
                }
                push_ctx_params(ctx, &args, &params);
                let result = run_with_ctx(&expr.clone(), ctx);
                pop_ctx_params(ctx);
                result
            }
            VmFunction::Native(func) => func(&params, ctx),
        }
    } else {
        Err(format!("function `{}` is unknown", name))
    }
}

fn run_tuple_exprs(params: &TupleType, ctx: &mut Box<dyn Lookable>) -> Result<TupleType, VmError>
{
    let mut list = vec![];
    for param in params {
        let result = run_with_ctx(&param, ctx)?;
        list.push(Expr::Value(result));
    }
    Ok(list)
}

fn push_ctx_params(ctx: &mut Box<dyn Lookable>, names: &TupleType, vals: &TupleType)
{
    use std::cell::RefCell;
    use std::rc::Rc;
    // TODO: probably not needed
    assert_eq!(names.len(), vals.len());
    let frame = names
        .iter()
        .zip(vals)
        .map(|(name, val)| {
            let expr = Box::new(val.clone());
            match name {
                Expr::Ref(name) => {
                    let func = VmFunction::Virtual((vec![], expr));
                    (name.clone(), Rc::new(RefCell::new(func)))
                }
                _ => unimplemented!(),
            }
        })
        .collect::<VmFrame>();
    ctx.push_frame(frame);
}

fn pop_ctx_params(ctx: &mut Box<dyn Lookable>)
{
    assert!(ctx.pop_frame());
}
