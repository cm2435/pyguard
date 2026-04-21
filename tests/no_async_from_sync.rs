mod helpers;
use helpers::lint_with_rule;

const RULE: &str = "no-async-from-sync";

#[test]
fn get_event_loop_in_sync_def() {
    let source = "def foo():\n    loop = asyncio.get_event_loop()\n    loop.run_until_complete(bar())\n";
    let d = lint_with_rule(source, RULE);
    // Both `asyncio.get_event_loop()` and `loop.run_until_complete(...)` should flag.
    assert_eq!(d.len(), 2);
    assert_eq!(d[0].line, 2);
    assert_eq!(d[1].line, 3);
}

#[test]
fn get_event_loop_in_async_def_ok() {
    let source = "async def foo():\n    loop = asyncio.get_event_loop()\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn asyncio_run_in_sync_def() {
    let source = "def foo():\n    asyncio.run(bar())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn asyncio_run_at_module_level_ok() {
    // Standard entrypoint pattern.
    let source = "if __name__ == \"__main__\":\n    asyncio.run(main())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn run_until_complete_in_sync_def() {
    let source = "def foo():\n    loop.run_until_complete(bar())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn new_event_loop_in_sync_def() {
    let source = "def foo():\n    loop = asyncio.new_event_loop()\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn ensure_future_in_sync_def() {
    let source = "def foo():\n    asyncio.ensure_future(bar())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn create_task_asyncio_in_sync_def() {
    let source = "def foo():\n    asyncio.create_task(bar())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn create_task_non_asyncio_object_ok() {
    // `scheduler.create_task(...)` is not asyncio — too ambiguous to flag.
    let source = "def foo():\n    scheduler.create_task(job)\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn obj_run_non_asyncio_ok() {
    // Bare `.run()` is common (threads, click, flask, subprocess). Don't flag.
    let source = "def foo():\n    thread.run()\n    app.run(host=\"0.0.0.0\")\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn sync_nested_in_async_flagged() {
    // Nearest enclosing function_definition wins. Sync helper defined inside
    // an async function is still sync — dispatching the loop from it is bad.
    let source = "async def outer():\n    def inner():\n        asyncio.run(bar())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}

#[test]
fn async_nested_in_sync_ok() {
    // Inverse: async helper defined inside a sync function. The async def
    // is the nearest scope — dispatch is fine there.
    let source = "def outer():\n    async def inner():\n        loop = asyncio.get_event_loop()\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn pure_sync_code_ok() {
    let source = "def foo():\n    x = 1\n    return x + 2\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn in_comment_ok() {
    let source = "# asyncio.run(main())\n# loop.run_until_complete(x)\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn method_def_in_sync_class() {
    // Sync method on a class — same rules apply.
    let source = "class Service:\n    def handle(self):\n        asyncio.run(self._work())\n";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}
