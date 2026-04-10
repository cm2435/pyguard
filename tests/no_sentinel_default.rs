mod helpers;
use helpers::lint_with_rule;

const RULE: &str = "no-sentinel-default";

// ═══════════════════════════════════════════════════════════════════════════
// Category 1: Nil UUID — UUID(int=0) / UUID("00000000-...")
// Real-world: fractal-os subgraph/service.py, workflow_level_serializer.py
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_uuid_int_zero() {
    let source = r#"
from uuid import UUID
def f(user_id: UUID = UUID(int=0)):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert!(d[0].message.contains("sentinel"));
}

#[test]
fn param_uuid_nil_string() {
    let source = r#"
from uuid import UUID
def f(user_id: UUID = UUID("00000000-0000-0000-0000-000000000000")):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_str_nil_uuid() {
    let source = r#"
def f(task_id: str = "00000000-0000-0000-0000-000000000000"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_optional_uuid_nil_instead_of_none() {
    // The exact anti-pattern: type is UUID | None but LLM puts a sentinel instead of None
    let source = r#"
from uuid import UUID
def f(user_id: UUID | None = UUID(int=0)):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_uuid_int_small_nonzero() {
    // UUID(int=1), UUID(int=42) etc. — still obviously synthetic
    let source = r#"
from uuid import UUID
def f(task_id: UUID = UUID(int=1)):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_uuid_int_small_hundred() {
    // Real-world: ma_gym_ref demos use UUID(int=100), UUID(int=101)
    let source = r#"
from uuid import UUID
def f(task_id: UUID = UUID(int=100)):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 2: Sequential digit UUIDs
// Real-world: fractal-os test_websocket_unit.py
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_sequential_uuid_string() {
    let source = r#"
def f(org_id: str = "12345678-1234-1234-1234-123456789012"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_sequential_uuid_constructor() {
    let source = r#"
from uuid import UUID
def f(org_id: UUID = UUID("12345678-1234-1234-1234-123456789012")):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_sequential_uuid_variant() {
    // 12345678-1234-5678-1234-567812345678
    let source = r#"
def f(org_id: str = "12345678-1234-5678-1234-567812345678"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 3: Repeated character UUIDs
// Real-world: arcane-dashboard dashboardFixtures.ts patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_repeated_digit_uuid() {
    // "11111111-1111-1111-1111-111111111111"
    let source = r#"
def f(cohort_id: str = "11111111-1111-1111-1111-111111111111"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_repeated_letter_uuid() {
    // "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
    let source = r#"
def f(criterion_id: str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_repeated_with_valid_version_nibble() {
    // "11111111-1111-4111-8111-111111111111" — has version 4 and variant bits but still fake
    let source = r#"
def f(run_id: str = "11111111-1111-4111-8111-111111111111"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_repeated_ff() {
    let source = r#"
def f(x: str = "ffffffff-ffff-ffff-ffff-ffffffffffff"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 4: RFC 4122 example UUID
// Real-world: ma_gym_ref marketing_campaign/workflow.py
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_rfc_example_uuid() {
    // This is the literal example UUID from RFC 4122 — a dead giveaway
    let source = r#"
from uuid import UUID
def f(owner_id: UUID = UUID("123e4567-e89b-12d3-a456-426614174000")):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_rfc_example_uuid_as_string() {
    let source = r#"
def f(owner_id: str = "123e4567-e89b-12d3-a456-426614174000"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 5: Near-nil / incrementing UUIDs (last group differs)
// Real-world: graphviz benchmark assets "00000000-0000-0000-0000-000000000001"
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_near_nil_uuid_incremented() {
    let source = r#"
def f(node_id: str = "00000000-0000-0000-0000-000000000001"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_near_nil_uuid_large_increment() {
    // "00000000-0000-0000-0000-0000000007d0" (2000 in hex)
    let source = r#"
def f(node_id: str = "00000000-0000-0000-0000-0000000007d0"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 6: Placeholder secret / token strings
// Real-world: ai8_agents/settings.py INNGEST_SIGNING_KEY: str = "DEADBEEF"
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_deadbeef() {
    let source = r#"
def f(signing_key: str = "DEADBEEF"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_placeholder_token() {
    let source = r#"
def f(api_key: str = "your-api-key-here"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_replace_me() {
    let source = r#"
def f(secret: str = "REPLACE_ME"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_changeme() {
    let source = r#"
def f(token: str = "CHANGEME"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_insert_here() {
    let source = r#"
def f(key: str = "INSERT_KEY_HERE"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_sk_prefix_api_key() {
    // OpenAI-style API key sentinel
    let source = r#"
def f(api_key: str = "sk-proj-abc123def456"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_fake_jwt() {
    // Real-world: test_websocket_unit.py token="fake-jwt"
    let source = r#"
def f(token: str = "fake-jwt"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 7: Placeholder URLs and emails
// Real-world: common LLM hallucination patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_example_com_url() {
    let source = r#"
def f(endpoint: str = "https://example.com/api/v1"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_example_org_url() {
    let source = r#"
def f(base_url: str = "https://api.example.org"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_example_email() {
    // Real-world: test_websocket_unit.py email="test@example.com"
    let source = r#"
def f(email: str = "test@example.com"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_user_at_example() {
    let source = r#"
def f(email: str = "user@example.com"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_placeholder_path() {
    let source = r#"
def f(output: str = "/path/to/file"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_placeholder_path_model() {
    let source = r#"
def f(model_path: str = "/path/to/model"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 8: Metasyntactic / generic placeholder strings
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn param_foo() {
    let source = r#"
def f(name: str = "foo"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_bar() {
    let source = r#"
def f(name: str = "bar"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_baz() {
    let source = r#"
def f(name: str = "baz"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_placeholder_literal() {
    let source = r#"
def f(value: str = "placeholder"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_lorem_ipsum() {
    let source = r#"
def f(text: str = "lorem ipsum"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_hello_world() {
    let source = r#"
def f(msg: str = "hello world"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn param_test_string() {
    let source = r#"
def f(label: str = "test_string"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 9: Class field sentinels (same patterns, model-field context)
// Real-world: Pydantic/SQLModel fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn class_field_uuid_int_zero() {
    let source = r#"
from uuid import UUID
class Task(BaseModel):
    id: UUID = UUID(int=0)
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 4);
}

#[test]
fn class_field_nil_uuid_string() {
    let source = r#"
class Config(BaseModel):
    org_id: str = "00000000-0000-0000-0000-000000000000"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn class_field_repeated_uuid() {
    let source = r#"
class Config(BaseModel):
    tenant_id: str = "11111111-1111-1111-1111-111111111111"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn class_field_deadbeef() {
    let source = r#"
class Settings(BaseModel):
    signing_key: str = "DEADBEEF"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn class_field_example_url() {
    let source = r#"
class ServiceConfig(BaseModel):
    base_url: str = "https://example.com"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn class_field_placeholder_email() {
    let source = r#"
class UserDefaults(BaseModel):
    email: str = "user@example.com"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn class_field_multiple_sentinels() {
    let source = r#"
class BadModel(BaseModel):
    org_id: str = "00000000-0000-0000-0000-000000000000"
    name: str = "foo"
    endpoint: str = "https://example.com"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 10: Multiple sentinels in one function signature
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_sentinel_params() {
    let source = r#"
from uuid import UUID
def process(
    task_id: UUID = UUID(int=0),
    org_id: str = "00000000-0000-0000-0000-000000000000",
    api_key: str = "REPLACE_ME",
):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 3);
}

#[test]
fn mixed_good_and_bad_params() {
    let source = r#"
from uuid import UUID
def process(
    data: dict,
    task_id: UUID = UUID(int=0),
    verbose: bool = False,
    name: str = "foo",
):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 2); // UUID(int=0) and "foo"
}

// ═══════════════════════════════════════════════════════════════════════════
// NEGATIVE TESTS — things that should NOT trigger
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ok_none_default() {
    let source = r#"
from uuid import UUID
def f(user_id: UUID | None = None):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_no_default() {
    let source = r#"
from uuid import UUID
def f(user_id: UUID):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_uuid4_factory() {
    let source = r#"
from uuid import uuid4
from pydantic import Field
class M(BaseModel):
    id: UUID = Field(default_factory=uuid4)
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_real_looking_string() {
    // A non-sentinel string default is fine (legitimate config value)
    let source = r#"
def f(mode: str = "production"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_meaningful_string_default() {
    let source = r#"
def f(format: str = "json"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_int_default() {
    let source = r#"
def f(retries: int = 3):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_bool_default() {
    let source = r#"
def f(verbose: bool = False):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_module_level_assignment() {
    // Module-level constants are intentional config, not sentinel defaults
    let source = r#"
DEFAULT_ORG_ID: str = "00000000-0000-0000-0000-000000000000"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_untyped_param() {
    // No type annotation — can't tell it's supposed to be a UUID
    let source = r#"
def f(user_id="00000000-0000-0000-0000-000000000000"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_genuine_uuid_in_string() {
    // A random-looking UUID that doesn't match any sentinel pattern
    let source = r#"
def f(default_tenant: str = "a3f7b2c1-4d8e-4f9a-b6c3-1e2d3f4a5b6c"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_uuid_int_large() {
    // UUID(int=very_large_number) is likely a real deterministic ID, not a sentinel
    // Threshold TBD — but 2**64 scale values are probably real
    let source = r#"
from uuid import UUID
def f(namespace_id: UUID = UUID(int=272437025437478349854831474661972096000)):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_in_comment() {
    let source = r#"
# def f(x: str = "00000000-0000-0000-0000-000000000000"):
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_in_docstring() {
    let source = r#"
def f():
    """Example: user_id = UUID("123e4567-e89b-12d3-a456-426614174000")"""
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_class_field_no_default() {
    let source = r#"
class M(BaseModel):
    id: UUID
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_class_field_none_default() {
    let source = r#"
class M(BaseModel):
    parent_id: UUID | None = None
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_class_field_real_string() {
    let source = r#"
class Config(BaseModel):
    region: str = "us-east-1"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn ok_async_function_no_sentinel() {
    let source = r#"
async def fetch(url: str = "https://internal.mycompany.com/api"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sentinel_in_async_function() {
    let source = r#"
from uuid import UUID
async def process(task_id: UUID = UUID(int=0)):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn sentinel_in_method() {
    let source = r#"
from uuid import UUID
class Service:
    def get_task(self, task_id: UUID = UUID(int=0)):
        pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn sentinel_single_quotes() {
    let source = "def f(x: str = '00000000-0000-0000-0000-000000000000'):\n    pass";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn sentinel_case_insensitive_placeholder() {
    let source = r#"
def f(x: str = "Placeholder"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn sentinel_case_insensitive_replace_me() {
    let source = r#"
def f(x: str = "replace_me"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn sentinel_deadbeef_lowercase() {
    let source = r#"
def f(key: str = "deadbeef"):
    pass
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn nested_class_field_sentinel() {
    let source = r#"
class Outer:
    class Inner(BaseModel):
        org_id: str = "00000000-0000-0000-0000-000000000000"
"#;
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}
