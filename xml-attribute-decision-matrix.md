# Decision Matrix: XML Attribute Support in quick-xml

## Problem Statement
**Problem**: Developers cannot use the same struct for both XML and JSON serialization because `#[serde(rename = "@fieldname")]` causes JSON validation failures with "@" in field names.

**Decision**: How should we implement `#[xml(attribute)]` to mark XML attributes without polluting JSON field names?

## Approaches

| **Criterion** | **Custom Derive Macro** | **Serde Attribute Extension** | **Runtime Format Detection** | **Dual Struct Pattern** |
|---------------|------------------------|------------------------------|----------------------------|------------------------|
| **Description** | Create `quick-xml-derive` crate with `#[derive(XmlSerialize)]` and `#[xml(attribute)]` | Proc macro transforms `#[xml(...)]` to `#[serde(rename = "@...")]` conditionally | Custom serializer strips/adds "@" based on format detection | Generate two structs (XML/JSON) from one definition with conversion |
| **Solves JSON validation** | 🟢 Clean separation - no "@" in JSON | 🟡 Complex - needs conditional compilation or feature flags | 🟡 Works but "@" still in field definition | 🟢 Complete separation of concerns |
| **Works with quick-xml infrastructure** | 🔴 Major rewrite - quick-xml built on serde | 🟡 Tricky integration - modifying serde attributes | 🟢 Minimal changes to existing code | 🟡 Need conversion layer between structs |
| **Clear semantic intent** | 🟢 `#[xml(attribute)]` is explicit | 🟢 `#[xml(attribute)]` is clear | 🔴 "@" prefix unclear for XML semantics | 🟢 Explicit struct purposes |
| **Backward compatibility** | 🔴 Breaking - new derive macro required | 🟡 Could support both old and new syntax | 🟢 Fully compatible - no API changes | 🟡 Requires migration to new pattern |
| **Ergonomic API** | 🟢 Clean, yaserde-like syntax | 🟡 Depends on implementation complexity | 🟡 Hidden magic, surprising behavior | 🔴 Verbose - two structs to maintain |
| **Code duplication** | 🟢 Single struct definition | 🟢 Single struct definition | 🟢 Single struct definition | 🔴 Conceptual duplication in generation |
| **IDE support** | 🟡 New macros need IDE updates | 🟡 Proc macro may confuse IDEs | 🟢 Standard serde, good support | 🟡 Generated code harder to navigate |
| **Performance** | 🟡 Compile-time cost of new derive | 🟡 Proc macro overhead at compile time | 🔴 Runtime overhead checking format | 🟡 Conversion cost between structs |
| **Maintenance burden** | 🔴 New crate to maintain, separate from serde | 🟡 Complex macro logic to maintain | 🟢 Simple runtime logic | 🟡 Code generation complexity |
| **Familiar patterns** | 🟢 Matches yaserde approach | 🟡 Novel approach, less precedent | 🟡 Unusual runtime detection | 🟡 Common but verbose pattern |
| **Serde feature compatibility** | 🔴 Lose serde ecosystem benefits | 🟢 Full serde compatibility | 🟢 Full serde compatibility | 🟢 Each struct fully serde-compatible |
| **Documentation clarity** | 🟢 Clear separation of concerns | 🟡 Need to explain transformation | 🟡 Runtime behavior needs explanation | 🟡 Pattern needs clear examples |

## Implementation Complexity Analysis

### Custom Derive Macro
**Effort**: High
- Build entire derive macro infrastructure
- Reimplement serde-like functionality
- Major breaking change for users

### Serde Attribute Extension
**Effort**: Medium-High
- Complex proc macro logic
- Must handle attribute transformation correctly
- Potential edge cases with serde interaction

### Runtime Format Detection
**Effort**: Low-Medium
- Implement custom serializer/deserializer wrappers
- Detection logic for format type
- No API changes needed

### Dual Struct Pattern
**Effort**: Medium
- Proc macro to generate two structs
- Implement conversion traits
- Clear but verbose for users

## Open Questions
- [ ] How much of quick-xml's existing code depends on serde's "@" convention?
- [ ] Would serde team accept patches for native XML attribute support?
- [ ] Can we detect serialization target (JSON vs XML) reliably at runtime?
- [ ] How many users currently rely on `#[serde(rename = "@...")]` pattern?
- [ ] Could we provide migration tools for existing codebases?

## Recommendation

**Recommended Approach: Serde Attribute Extension (Approach 2) with migration path**

### Rationale
1. **Preserves serde ecosystem** - Users keep all serde benefits (other formats, validation, etc.)
2. **Matches user expectations** - `#[xml(attribute)]` syntax as requested
3. **Feasible implementation** - While complex, it's a bounded problem
4. **Migration friendly** - Can support both old and new syntax during transition

### Implementation Strategy
1. **Phase 1**: Create proc macro that recognizes `#[xml(attribute)]`
2. **Phase 2**: Transform to appropriate serde attributes based on feature flags
3. **Phase 3**: Provide migration guide and tooling
4. **Phase 4**: Eventually deprecate "@" pattern

### Key Design Decisions
- Use feature flags: `xml-derive` for new syntax support
- Generate `#[serde(rename = "@field")]` only when serializing to XML
- For JSON, use original field name or `#[xml(rename = "...")]` if specified
- Maintain backward compatibility with deprecation warnings

## Tradeoffs

**What we're accepting:**
- Increased compile-time complexity from proc macros
- Some "magic" in attribute transformation
- Potential IDE confusion until tooling catches up

**What we're gaining:**
- Clean struct definitions that work for both XML and JSON
- Preservation of serde ecosystem benefits
- Clear semantic intent with `#[xml(attribute)]`
- Gradual migration path for existing users

**What we're avoiding:**
- Complete rewrite of quick-xml's serde integration (Approach 1)
- Runtime performance overhead (Approach 3)
- Verbose dual-struct patterns (Approach 4)

## Alternative Consideration

If Approach 2 proves too complex, **Runtime Format Detection (Approach 3)** becomes the pragmatic choice:
- Minimal code changes
- Full backward compatibility
- Simple implementation

The runtime overhead might be acceptable given that serialization is typically not the bottleneck in XML processing.

## Success Metrics
- Zero breaking changes for existing users
- Clean JSON output without "@" prefixes
- Maintain or improve serialization performance
- Clear migration path with tooling support