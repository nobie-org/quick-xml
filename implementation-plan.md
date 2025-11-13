# Simplified Implementation Plan for #[xml(attribute)]

## Overview

Add support for `#[xml(attribute)]` without requiring a proc macro initially. Instead, leverage serde's existing infrastructure and quick-xml's runtime attribute detection.

## Core Insight

Serde already provides field attributes through its `rename` system. We can:
1. Detect `#[xml(attribute)]` patterns in field names during deserialization
2. Use a special naming convention that serde passes through
3. Keep full backward compatibility with `@` prefix

## Implementation Approach

### Step 1: Define Special Marker

Instead of a proc macro, use a naming convention that serde will preserve:

```rust
#[derive(Deserialize)]
struct User {
    // New pattern: Use a special prefix that we detect
    #[serde(rename = "@@xml:attr:id")]
    id: String,

    // Old pattern: Still works
    #[serde(rename = "@name")]
    name: String,
}
```

Wait, this is ugly. Let's think differently...

### Better Approach: Runtime Attribute Registry

Since we can't easily get compile-time information without a proc macro, let's use a runtime registry pattern:

```rust
// User defines their struct normally
#[derive(Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

// User registers which fields are attributes
impl User {
    fn xml_attributes() -> &'static [&'static str] {
        &["id", "email"]
    }
}
```

But this requires users to implement a trait, which is not much better than the current solution.

### Actual Simple Solution: Hybrid Detection

The cleanest approach without proc macros is to support BOTH patterns in the deserializer:

1. Keep `@` prefix detection (current behavior)
2. Add detection for fields that are known to be attributes from context
3. Use deserializer state to track what we're expecting

Here's the key insight: When deserializing, we already know from the XML structure what are attributes vs elements. We can use this information!

## Revised Implementation Plan

### Phase 1: Track Attribute Names During Parsing

```rust
// In src/de/mod.rs
pub struct Deserializer<'de, R> {
    // ... existing fields ...

    /// Attributes found in current element (NEW)
    current_attributes: HashSet<String>,
}

// When we parse a start tag:
impl<'de, R> Deserializer<'de, R> {
    fn read_start_tag(&mut self, tag: &BytesStart) -> Result<(), DeError> {
        // Collect all attribute names (without @ prefix)
        self.current_attributes.clear();
        for attr in tag.attributes() {
            let attr = attr?;
            let name = self.decode_name(attr.key)?;
            self.current_attributes.insert(name);
        }
        // ... rest of processing
    }
}
```

### Phase 2: Smart Field Resolution

When deserializing struct fields, check BOTH patterns:

```rust
// In src/de/map.rs
impl<'de, 'a, R> MapAccess<'de> for ElementMapAccess<'de, 'a, R> {
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, DeError> {
        // Try to get next field name
        let field_name = self.get_next_field_name()?;

        // Check if this field corresponds to an attribute
        let is_attribute = self.is_attribute_field(&field_name)?;

        if is_attribute {
            // Look for attribute value
            self.deserialize_attribute(field_name, seed)
        } else {
            // Look for element value
            self.deserialize_element(field_name, seed)
        }
    }

    fn is_attribute_field(&self, field_name: &str) -> bool {
        // Method 1: Check for @ prefix (backward compatibility)
        if field_name.starts_with('@') {
            return true;
        }

        // Method 2: Check if XML has an attribute with this name
        // This is the KEY: we already KNOW what attributes exist!
        if self.de.current_attributes.contains(field_name) {
            return true;
        }

        false
    }
}
```

### Phase 3: Bidirectional Mapping

Support both ways of accessing the same attribute:

```rust
// User can write either:
#[derive(Deserialize)]
struct UserOldStyle {
    #[serde(rename = "@id")]
    id: String,
}

// OR:
#[derive(Deserialize)]
struct UserNewStyle {
    id: String,  // No rename needed!
}

// Both work with: <user id="123"/>
```

The deserializer will:
1. See attribute "id" in XML
2. Try to match against field "id" OR field "@id"
3. Deserialize to whichever matches

### Phase 4: Handle Ambiguity

What if there's both an attribute and element with same name?

```xml
<user id="123">
    <id>456</id>
</user>
```

Resolution rules:
1. Fields with `@` prefix ONLY match attributes
2. Fields without prefix prefer elements but fall back to attributes
3. User can force attribute-only with `@` prefix

## Minimal Code Changes

### File: src/de/mod.rs

```rust
pub struct Deserializer<'de, R: XmlRead<'de>> {
    // ... existing fields ...

    /// Attributes in current element (without @ prefix)
    pub(crate) current_attributes: HashSet<String>,
}

impl<'de, R: XmlRead<'de>> Deserializer<'de, R> {
    pub fn new(reader: R) -> Self {
        Deserializer {
            // ... existing fields ...
            current_attributes: HashSet::new(),
        }
    }
}
```

### File: src/de/map.rs

```rust
impl<'de, 'a, R: XmlRead<'de>> ElementMapAccess<'de, 'a, R> {
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, DeError>
    where
        K: DeserializeSeed<'de>,
    {
        // First, try attributes
        if let Some((key, value)) = self.iter.next(&mut self.de.key_buf)? {
            let attr_name = decode_attr_name(key)?;

            // Try both with and without @ prefix
            if self.try_match_field(&attr_name, seed)? {
                self.source = ValueSource::Attribute(value.unwrap_or_default());
                return Ok(Some(seed.deserialize(attr_name)?));
            }

            // Also try with @ prefix for backward compatibility
            let prefixed_name = format!("@{}", attr_name);
            if self.try_match_field(&prefixed_name, seed)? {
                self.source = ValueSource::Attribute(value.unwrap_or_default());
                return Ok(Some(seed.deserialize(prefixed_name)?));
            }
        }

        // Then try elements (existing logic)
        // ...
    }
}
```

## Benefits of This Approach

1. **No proc macro needed**: Works with plain `#[derive(Deserialize)]`
2. **Zero breaking changes**: All existing code continues to work
3. **Natural migration**: Users can gradually remove `@` prefixes
4. **Dual format friendly**: Same struct works for JSON and XML
5. **Simple implementation**: Mostly changes to field matching logic

## Testing

```rust
#[test]
fn test_attribute_without_prefix() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct User {
        id: String,      // No @ prefix!
        name: String,    // This will be an element
    }

    let xml = r#"<user id="123"><name>Alice</name></user>"#;
    let user: User = from_str(xml).unwrap();

    assert_eq!(user.id, "123");
    assert_eq!(user.name, "Alice");
}

#[test]
fn test_backward_compatibility() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct User {
        #[serde(rename = "@id")]
        id: String,
        name: String,
    }

    let xml = r#"<user id="123"><name>Alice</name></user>"#;
    let user: User = from_str(xml).unwrap();

    assert_eq!(user.id, "123");
    assert_eq!(user.name, "Alice");
}

#[test]
fn test_mixed_style() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct User {
        #[serde(rename = "@id")]
        id: String,        // Old style
        email: String,     // New style - will match attribute
        name: String,      // Will match element
    }

    let xml = r#"<user id="123" email="alice@example.com"><name>Alice</name></user>"#;
    let user: User = from_str(xml).unwrap();

    assert_eq!(user.id, "123");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.name, "Alice");
}
```

## Summary

This approach provides `#[xml(attribute)]` functionality without the syntax:
- Fields automatically match XML attributes if they have the same name
- No need for `@` prefix in field names
- Full backward compatibility maintained
- Implementation requires minimal changes to existing code
- Natural and intuitive for users

The key insight: **We already know what's an attribute from the XML structure itself!** We don't need users to tell us - we can figure it out at runtime by matching field names to attribute names.