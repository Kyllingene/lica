use std::collections::HashMap;

#[derive(Default)]
struct Context {
    globals: HashMap<String, Expr>,
    display: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct Lambda {
    name: Option<String>,
    arg: String,
    body: Expr,
}

#[derive(Debug, Clone)]
struct Apply {
    lambda: Expr,
    arg: Expr,
}

#[derive(Debug, Clone)]
enum Expr {
    Nil,
    Var(String),
    Lambda(Box<Lambda>),
    Apply(Box<Apply>),
    Builtin(fn(Expr, &mut Context) -> Expr),
    Halt,
}

impl Lambda {
    fn apply(mut self, arg: Expr) -> Expr {
        self.body.substitute(&self.arg, arg);
        self.body
    }
}

impl Expr {
    fn apply(self, arg: Expr, ctx: &mut Context) -> Expr {
        match self {
            Expr::Lambda(l) => l.apply(arg),
            Expr::Builtin(f) => f(arg, ctx),
            _ => panic!("tried to apply a non-lambda"),
        }
    }

    fn substitute(&mut self, name: &str, expr: Expr) {
        match self {
            Expr::Var(s) if s == name => {
                *self = expr;
            }
            Expr::Lambda(l) if l.arg != name => l.body.substitute(name, expr),
            Expr::Apply(a) => {
                if let Expr::Var(v) = &a.lambda {
                    if v == name {
                        a.lambda = expr.clone();
                    }
                }
                a.arg.substitute(name, expr);
            }
            _ => {}
        }
    }

    fn eval(self, ctx: &mut Context) -> Expr {
        match self {
            Expr::Var(v) => ctx.globals.get(&v).unwrap().clone(),
            Expr::Apply(a) => a.lambda.eval(ctx).apply(a.arg.eval(ctx), ctx).eval(ctx),
            Expr::Halt => std::process::exit(0),
            other => other,
        }
    }
}

fn builtins() -> Context {
    let mut context = Context::default();

    context.globals.insert("halt".to_string(), Expr::Halt);
    context.globals.insert(
        "?".to_string(),
        Expr::Builtin(|expr, ctx| {
            match expr {
                Expr::Nil => println!("nil"),
                Expr::Lambda(l) => {
                    if let Some(v) = &l.name {
                        if let Some(s) = ctx.display.get(v) {
                            println!("{s}");
                        } else {
                            println!("<lambda {v}>");
                        }
                    } else {
                        println!("<lambda>");
                    }
                }
                Expr::Builtin(_) => println!("<builtin>"),
                _ => {}
            }
            Expr::Nil
        }),
    );

    context
}

fn main() {
    let mut ctx = builtins();
    let t = Expr::Lambda(Box::new(Lambda {
        name: Some("true".into()),
        arg: "x".into(),
        body: Expr::Lambda(Box::new(Lambda {
            name: None,
            arg: "y".into(),
            body: Expr::Var("x".into()),
        })),
    }));

    let f = Expr::Lambda(Box::new(Lambda {
        name: Some("false".into()),
        arg: "x".into(),
        body: Expr::Lambda(Box::new(Lambda {
            name: None,
            arg: "y".into(),
            body: Expr::Var("y".into()),
        })),
    }));

    ctx.globals.insert("true".into(), t);
    ctx.globals.insert("false".into(), f);
    ctx.display.insert("true".into(), "true".into());
    ctx.display.insert("false".into(), "false".into());

    Expr::Apply(Box::new(Apply {
        lambda: Expr::Var("?".into()),
        arg: Expr::Apply(Box::new(Apply {
            lambda: Expr::Apply(Box::new(Apply {
                lambda: Expr::Var("true".into()),
                arg: Expr::Var("false".into()),
            })),
            arg: Expr::Var("false".into()),
        })),
    })).eval(&mut ctx);
}
