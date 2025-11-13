# Schema/Selection Analysis: XML Attribute Support in quick-xml

## Current Problem

quick-xml currently conflates two orthogonal concerns:

1. **Schema** (context-free): What IS an XML attribute vs element - a structural property of XML
2. **Selection** (context-specific): HOW users choose to mark attributes - syntax preference

Currently, these are incorrectly coupled through the `@` prefix convention:

```rust
// Current: Optionality baked into schema definition
#[derive(Deserialize)]
struct User {
    #[serde(rename = "@id")]  // Forces "@" prefix, mixing schema with selection
    id: String,
    name: String,              // Element by default
}
```

The problem: The `@` prefix serves dual purpose:
- **Schema marker**: Indicates this field maps to an XML attribute
- **Selection syntax**: Forces a specific naming convention

This violates Rich Hickey's principle: "Nothing is inherently Maybe" - whether something is an attribute is not optional, it IS an attribute. The choice of syntax to mark it is separate.

## Schemas (Shapes Only)

### XML Structure Schema

```rust
// What CAN be present in XML
enum XmlFieldKind {
    Attribute,    // Field maps to XML attribute
    Element,      // Field maps to XML element
    Text,         // Field maps to text content ($text)
    Value,        // Field maps to polymorphic content ($value)
}

// The structural shape of XML mapping
struct XmlFieldSchema {
    name: String,           // Field name in Rust
    xml_name: String,       // Name in XML (without @ prefix)
    kind: XmlFieldKind,     // What it IS in XML
}

// Complete mapping schema
struct XmlStructSchema {
    rust_name: String,
    xml_name: String,
    fields: Vec<XmlFieldSchema>,
}
```

**Purpose**: Describes the structural relationship between Rust types and XML, independent of syntax choices.

### Serde Integration Schema

```rust
// How serde sees the data model
struct SerdeFieldSchema {
    rust_name: String,
    serialized_name: String,  // What serde uses for matching
    field_type: Type,
}

struct SerdeStructSchema {
    struct_name: String,
    fields: Vec<SerdeFieldSchema>,
}
```

**Purpose**: Describes what serde expects during serialization/deserialization, independent of XML structure.

## Selections (Context Requirements)

### Context: User Chooses Old Syntax

**Needs**:
```rust
// User wants to use existing @ convention
struct OldStyleSelection {
    attribute_prefix: "@",     // Use @ to mark attributes
    field_rename: true,        // Must rename fields with serde
}

// Results in:
#[derive(Deserialize)]
struct User {
    #[serde(rename = "@id")]
    id: String,
}
```

**Rationale**: Backward compatibility, existing codebases, familiar pattern

### Context: User Chooses New Syntax

**Needs**:
```rust
// User wants clean dual-format support
struct NewStyleSelection {
    xml_attribute_marker: "#[xml(attribute)]",  // Explicit attribute marking
    field_rename: false,                        // No field renaming needed
}

// Results in:
#[derive(Deserialize)]
struct User {
    #[xml(attribute)]
    id: String,  // Clean field name, no @ prefix
}
```

**Rationale**: Cleaner for dual JSON/XML formats, no field name pollution

### Context: Deserialization Runtime

**Needs**:
```rust
struct DeserializationRequirements {
    // Must determine if field is attribute
    is_attribute_check: Box<dyn Fn(&str) -> bool>,

    // Must map XML names to field names
    field_name_mapper: Box<dyn Fn(&str, bool) -> String>,

    // Must handle both old and new patterns
    compatibility_mode: CompatibilityMode,
}

enum CompatibilityMode {
    OldOnly,       // Only @ prefix (current)
    NewOnly,       // Only #[xml] attributes
    Both,          // Support both patterns (migration period)
}
```

**Rationale**: Runtime needs to know how to identify and map attributes regardless of syntax choice

## Benefits

1. **Schema Reuse**: Single XML structure definition works with multiple syntax selections
2. **Clear Requirements**: Each context (old style, new style, runtime) states exactly what it needs
3. **Compatible Changes**:
   - Adding new syntax: ✅ Compatible (additive)
   - Supporting old syntax: ✅ Compatible (preserved)
   - Migrating between: ✅ Compatible (both work)
4. **Deep Specs**: Can specify attribute requirements independently of element requirements

## Implementation Strategy

### Phase 1: Separate Internal Representation

Create clean separation between XML structure and serde naming:

```rust
// In src/de/field_kind.rs (NEW FILE)
pub enum FieldKind {
    Attribute { xml_name: String },
    Element { xml_name: String },
    Text,
    Value,
}

impl FieldKind {
    /// Determine field kind from serde field name (old style)
    pub fn from_serde_name(name: &str) -> (Self, String) {
        if name.starts_with('@') {
            (Self::Attribute {
                xml_name: name[1..].to_string()
            }, name.to_string())
        } else if name == "$text" {
            (Self::Text, name.to_string())
        } else if name == "$value" {
            (Self::Value, name.to_string())
        } else {
            (Self::Element {
                xml_name: name.to_string()
            }, name.to_string())
        }
    }

    /// Check if this is an attribute field (NEW)
    pub fn is_attribute(&self) -> bool {
        matches!(self, Self::Attribute { .. })
    }
}
```

### Phase 2: Add Metadata Collection

Extend deserializer to collect field metadata:

```rust
// In src/de/mod.rs (MODIFY)
pub struct Deserializer<'de, R> {
    // ... existing fields ...

    /// Field metadata collected during deserialization (NEW)
    field_metadata: HashMap<String, FieldKind>,
}

impl<'de, R> Deserializer<'de, R> {
    /// Register field with its kind (NEW)
    pub fn register_field(&mut self, rust_name: String, kind: FieldKind) {
        self.field_metadata.insert(rust_name, kind);
    }

    /// Check if field is an attribute by metadata (NEW)
    pub fn is_attribute_field(&self, name: &str) -> bool {
        self.field_metadata
            .get(name)
            .map(|k| k.is_attribute())
            .unwrap_or(false)
    }
}
```

### Phase 3: Support #[xml(attribute)] Collection

Add proc macro to collect XML attributes at compile time:

```rust
// In quick-xml-derive/src/lib.rs (NEW CRATE)
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta};

#[proc_macro_derive(XmlDeserialize, attributes(xml))]
pub fn derive_xml_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Collect fields marked with #[xml(attribute)]
    let xml_attributes = extract_xml_attributes(&input);

    // Generate metadata registration
    let register_calls = xml_attributes.iter().map(|field_name| {
        quote! {
            deserializer.register_field(
                stringify!(#field_name).to_string(),
                FieldKind::Attribute {
                    xml_name: stringify!(#field_name).to_string()
                }
            );
        }
    });

    // Generate implementation
    let name = &input.ident;
    let expanded = quote! {
        impl XmlDeserialize for #name {
            fn register_metadata(deserializer: &mut Deserializer) {
                #(#register_calls)*
            }
        }
    };

    TokenStream::from(expanded)
}
```

### Phase 4: Runtime Selection

Modify deserializer to check both patterns:

```rust
// In src/de/map.rs (MODIFY)
impl<'de, 'a, R> MapAccess<'de> for ElementMapAccess<'de, 'a, R> {
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, DeError>
    where
        K: DeserializeSeed<'de>,
    {
        // ... existing code ...

        if let Some((key, value)) = self.iter.next(&mut self.de.key_buf)? {
            // Check for attribute using BOTH methods
            let is_attribute = {
                // Method 1: Check @ prefix (old style)
                let has_prefix = self.de.key_buf.starts_with('@');

                // Method 2: Check metadata (new style)
                let field_name = if has_prefix {
                    &self.de.key_buf[1..]
                } else {
                    &self.de.key_buf
                };
                let has_metadata = self.de.is_attribute_field(field_name);

                has_prefix || has_metadata
            };

            if is_attribute {
                // Handle as attribute
                self.source = ValueSource::Attribute(value.unwrap_or_default());

                // Add @ prefix only if not already present (for serde compatibility)
                if !self.de.key_buf.starts_with('@') {
                    self.de.key_buf.insert(0, '@');
                }

                // ... rest of attribute handling ...
            }
        }
    }
}
```

### Phase 5: Integration Without Breaking Changes

1. **Keep existing @ behavior**: All current code continues to work
2. **Add new #[xml(attribute)]**: Opt-in for new style
3. **Support both simultaneously**: Migration period friendly

```rust
// Both styles work in same codebase:
#[derive(Deserialize)]
struct MixedStyle {
    #[serde(rename = "@id")]  // Old style - still works
    id: String,

    #[xml(attribute)]          // New style - also works
    class: String,

    name: String,              // Element - unchanged
}
```

## Migration Path

### Stage 1: Internal Refactoring (No Breaking Changes)
1. Add `FieldKind` enum internally
2. Refactor attribute detection to use `FieldKind`
3. Ensure all tests pass with no external changes

### Stage 2: Add New Feature (Additive Only)
1. Add `#[xml(attribute)]` support alongside existing
2. Document new feature as experimental/beta
3. Both old and new patterns work

### Stage 3: Encourage Migration (Documentation)
1. Update examples to show new pattern
2. Add migration guide
3. Keep old pattern fully supported

### Stage 4: Long-term (Years Later)
1. Consider deprecation warnings for @ pattern
2. Provide automated migration tool
3. Never break existing code

## Testing Strategy

```rust
#[test]
fn test_old_style_still_works() {
    #[derive(Deserialize)]
    struct OldStyle {
        #[serde(rename = "@id")]
        id: String,
    }

    let xml = r#"<root id="123"/>"#;
    let result: OldStyle = from_str(xml).unwrap();
    assert_eq!(result.id, "123");
}

#[test]
fn test_new_style_works() {
    #[derive(Deserialize, XmlDeserialize)]
    struct NewStyle {
        #[xml(attribute)]
        id: String,
    }

    let xml = r#"<root id="123"/>"#;
    let result: NewStyle = from_str(xml).unwrap();
    assert_eq!(result.id, "123");
}

#[test]
fn test_both_styles_together() {
    #[derive(Deserialize, XmlDeserialize)]
    struct MixedStyle {
        #[serde(rename = "@old_id")]
        old_id: String,

        #[xml(attribute)]
        new_id: String,
    }

    let xml = r#"<root old_id="123" new_id="456"/>"#;
    let result: MixedStyle = from_str(xml).unwrap();
    assert_eq!(result.old_id, "123");
    assert_eq!(result.new_id, "456");
}
```

## Key Files to Modify

1. **src/de/field_kind.rs** (NEW): Define `FieldKind` enum
2. **src/de/mod.rs**: Add metadata collection to `Deserializer`
3. **src/de/map.rs**: Update attribute detection logic
4. **src/de/key.rs**: Modify `QNameDeserializer` to check metadata
5. **quick-xml-derive/** (NEW CRATE): Proc macro for `#[xml(attribute)]`
6. **Cargo.toml**: Add optional dependency on derive crate

## Summary

This design achieves complete separation of concerns:

- **Schema** (what IS an attribute) is separated from **Selection** (how users MARK attributes)
- Old `@` pattern continues working (100% backward compatible)
- New `#[xml(attribute)]` pattern is cleaner for dual-format use
- Users can mix both patterns during migration
- No breaking changes, purely additive feature
- Implementation is incremental and testable at each stage