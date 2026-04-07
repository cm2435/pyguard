mod helpers;
use helpers::lint_with_rule;

#[test]
fn under_limit_ok() {
    let source = "def f(a, b, c):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 0);
}

#[test]
fn at_limit_ok() {
    let source = "def f(a, b, c, d, e, f, g, h):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 0);
}

#[test]
fn over_limit() {
    let source = "def f(a, b, c, d, e, f, g, h, i):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn self_not_counted() {
    // self + 8 real params = 8 counted, should be ok
    let source = "class C:\n    def m(self, a, b, c, d, e, f, g, h):\n        pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 0);
}

#[test]
fn cls_not_counted() {
    let source = "class C:\n    @classmethod\n    def m(cls, a, b, c, d, e, f, g, h):\n        pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 0);
}

#[test]
fn self_plus_too_many() {
    // self + 9 real params = 9 counted, over limit
    let source = "class C:\n    def m(self, a, b, c, d, e, f, g, h, i):\n        pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
}

#[test]
fn typed_params_counted() {
    let source = "def f(a: int, b: str, c: float, d: bool, e: list, f: dict, g: set, h: tuple, i: bytes):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
}

#[test]
fn default_params_counted() {
    let source = "def f(a=1, b=2, c=3, d=4, e=5, f=6, g=7, h=8, i=9):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
}

#[test]
fn star_args_counted() {
    let source = "def f(a, b, c, d, e, f, g, h, *args):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
}

#[test]
fn kwargs_counted() {
    let source = "def f(a, b, c, d, e, f, g, h, **kwargs):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
}

#[test]
fn async_function() {
    let source = "async def f(a, b, c, d, e, f, g, h, i):\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 1);
}

#[test]
fn zero_params_ok() {
    let source = "def f():\n    pass";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 0);
}

#[test]
fn lambda_not_flagged() {
    // Lambdas use lambda node, not function_definition
    let source = "f = lambda a, b, c, d, e, f, g, h, i: None";
    let d = lint_with_rule(source, "max-function-params");
    assert_eq!(d.len(), 0);
}
