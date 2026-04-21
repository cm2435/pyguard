use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

// Attribute names that are unambiguously asyncio loop-dispatch.
// Matching on attribute alone keeps the rule resilient to `import asyncio as aio`.
const ALWAYS_BANNED_ATTRS: &[&str] = &[
    "get_event_loop",
    "new_event_loop",
    "run_until_complete",
    "ensure_future",
];

// Attribute names that are common outside asyncio (`app.run`, `scheduler.create_task`).
// Only flag when the object is literally `asyncio`.
const ASYNCIO_ONLY_ATTRS: &[&str] = &["run", "create_task"];

pub struct NoAsyncFromSync;

impl Rule for NoAsyncFromSync {
    fn name(&self) -> &'static str {
        "no-async-from-sync"
    }

    fn help(&self) -> &'static str {
        "Dispatching an async coroutine from a sync function via \
         `asyncio.get_event_loop()`, `asyncio.run()`, `loop.run_until_complete()`, \
         `asyncio.ensure_future()`, or `asyncio.create_task()` either deadlocks \
         (when a loop is already running), silently creates a new loop that \
         never runs tasks scheduled elsewhere, or raises `RuntimeError`. \
         Make the caller `async def` and `await` the coroutine, or move the \
         sync/async boundary to a single well-defined entrypoint \
         (`asyncio.run(main())` at `if __name__ == \"__main__\"`). If this is \
         a deliberate sync/async bridge (e.g. `nest_asyncio`, Jupyter, Django \
         ORM adapter), suppress with `# slopcop: ignore[no-async-from-sync]`."
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(func) = node.child_by_field_name("function") else {
            return;
        };
        if func.kind() != "attribute" {
            return;
        }
        let Some(attr) = func.child_by_field_name("attribute") else {
            return;
        };
        let attr_name = attr.utf8_text(source).unwrap_or("");

        let matched_name = if ALWAYS_BANNED_ATTRS.contains(&attr_name) {
            attr_name
        } else if ASYNCIO_ONLY_ATTRS.contains(&attr_name) {
            let Some(obj) = func.child_by_field_name("object") else {
                return;
            };
            if obj.kind() != "identifier" || obj.utf8_text(source).unwrap_or("") != "asyncio" {
                return;
            }
            attr_name
        } else {
            return;
        };

        // Find the nearest enclosing function_definition. No enclosing function →
        // module-level call (fine: the `if __name__ == "__main__": asyncio.run(...)`
        // entrypoint pattern).
        let Some(func_def) = ancestors
            .iter()
            .rev()
            .find(|a| a.kind() == "function_definition")
        else {
            return;
        };

        // If the enclosing def is `async def`, the user has proper options
        // (`await`, `asyncio.get_running_loop`) — this rule doesn't apply.
        let is_async = func_def
            .child(0)
            .map(|c| c.kind() == "async")
            .unwrap_or(false);
        if is_async {
            return;
        }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-async-from-sync",
            severity: crate::rules::Severity::Error,
            message: format!(
                "Avoid `{matched_name}()` inside a sync function; make the caller `async def` or move the sync/async boundary to an entrypoint"
            ),
        });
    }
}
