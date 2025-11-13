//! Tests for the `serde_helpers::attribute` module that provides an alternative
//! to `#[serde(rename = "@field")]` for marking XML attributes.

use pretty_assertions::assert_eq;
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

/// Test basic attribute serialization and deserialization
#[test]
fn test_attribute_helper_basic() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "user")]
    struct User {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        id: u32,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        class: String,

        name: String,
    }

    let user = User {
        id: 123,
        class: "admin".to_string(),
        name: "Alice".to_string(),
    };

    // Serialize to XML
    let xml = to_string(&user).unwrap();
    eprintln!("Serialized XML: {}", xml);
    assert_eq!(
        xml,
        r#"<user id="123" class="admin"><name>Alice</name></user>"#
    );

    // Deserialize from XML
    eprintln!("Deserializing XML: {}", xml);
    let deserialized: User = from_str(&xml).unwrap();
    assert_eq!(deserialized, user);
}

/// Test attribute helper with different data types
#[test]
fn test_attribute_helper_types() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "config")]
    struct Config {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        enabled: bool,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        count: i32,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        ratio: f64,

        description: String,
    }

    let config = Config {
        enabled: true,
        count: -42,
        ratio: 2.5,
        description: "Test config".to_string(),
    };

    // Serialize to XML
    let xml = to_string(&config).unwrap();
    assert_eq!(
        xml,
        r#"<config enabled="true" count="-42" ratio="2.5"><description>Test config</description></config>"#
    );

    // Deserialize from XML
    let deserialized: Config = from_str(&xml).unwrap();
    assert_eq!(deserialized, config);
}

/// Test backward compatibility: old @-prefix style still works
#[test]
fn test_backward_compatibility_old_style() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "item")]
    struct Item {
        #[serde(rename = "@id")]
        id: u32,

        #[serde(rename = "@category")]
        category: String,

        value: String,
    }

    let item = Item {
        id: 456,
        category: "books".to_string(),
        value: "Novel".to_string(),
    };

    // Serialize to XML
    let xml = to_string(&item).unwrap();
    assert_eq!(
        xml,
        r#"<item id="456" category="books"><value>Novel</value></item>"#
    );

    // Deserialize from XML
    let deserialized: Item = from_str(&xml).unwrap();
    assert_eq!(deserialized, item);
}

/// Test mixing old and new styles in the same struct
#[test]
fn test_mixed_styles() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "product")]
    struct Product {
        // Old style with @ prefix
        #[serde(rename = "@id")]
        id: u32,

        // New style with attribute helper
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        sku: String,

        // Regular element
        name: String,
    }

    let product = Product {
        id: 789,
        sku: "SKU123".to_string(),
        name: "Widget".to_string(),
    };

    // Serialize to XML
    let xml = to_string(&product).unwrap();
    assert_eq!(
        xml,
        r#"<product id="789" sku="SKU123"><name>Widget</name></product>"#
    );

    // Deserialize from XML
    let deserialized: Product = from_str(&xml).unwrap();
    assert_eq!(deserialized, product);
}

/// Test with optional attributes
#[test]
fn test_optional_attributes() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "node")]
    struct Node {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        id: String,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        optional: Option<String>,

        content: String,
    }

    // With optional attribute present
    let node_with = Node {
        id: "n1".to_string(),
        optional: Some("value".to_string()),
        content: "data".to_string(),
    };

    let xml_with = to_string(&node_with).unwrap();
    assert_eq!(
        xml_with,
        r#"<node id="n1" optional="value"><content>data</content></node>"#
    );

    let deserialized_with: Node = from_str(&xml_with).unwrap();
    assert_eq!(deserialized_with, node_with);

    // With optional attribute absent
    let node_without = Node {
        id: "n2".to_string(),
        optional: None,
        content: "data".to_string(),
    };

    let xml_without = to_string(&node_without).unwrap();
    assert_eq!(
        xml_without,
        r#"<node id="n2"><content>data</content></node>"#
    );

    // Deserialize - optional field should be None
    let deserialized_without: Node = from_str(&xml_without).unwrap();
    assert_eq!(deserialized_without.id, "n2");
    assert_eq!(deserialized_without.optional, None);
    assert_eq!(deserialized_without.content, "data");
}

/// Test with nested structs
#[test]
fn test_nested_structs() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "address")]
    struct Address {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        country: String,

        city: String,
        street: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "person")]
    struct Person {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        id: u32,

        name: String,
        address: Address,
    }

    let person = Person {
        id: 100,
        name: "Bob".to_string(),
        address: Address {
            country: "US".to_string(),
            city: "NYC".to_string(),
            street: "5th Ave".to_string(),
        },
    };

    // Serialize to XML
    let xml = to_string(&person).unwrap();
    assert_eq!(
        xml,
        r#"<person id="100"><name>Bob</name><address country="US"><city>NYC</city><street>5th Ave</street></address></person>"#
    );

    // Deserialize from XML
    let deserialized: Person = from_str(&xml).unwrap();
    assert_eq!(deserialized, person);
}

/// Test with Vec of elements
#[test]
fn test_vec_with_attributes() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "tag")]
    struct Tag {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        name: String,

        #[serde(rename = "$text")]
        value: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "document")]
    struct Document {
        #[serde(rename = "tag")]
        tags: Vec<Tag>,
    }

    let doc = Document {
        tags: vec![
            Tag {
                name: "author".to_string(),
                value: "Alice".to_string(),
            },
            Tag {
                name: "version".to_string(),
                value: "1.0".to_string(),
            },
        ],
    };

    // Serialize to XML
    let xml = to_string(&doc).unwrap();
    assert_eq!(
        xml,
        r#"<document><tag name="author">Alice</tag><tag name="version">1.0</tag></document>"#
    );

    // Deserialize from XML
    let deserialized: Document = from_str(&xml).unwrap();
    assert_eq!(deserialized, doc);
}

/// Test attribute with rename
#[test]
fn test_attribute_with_rename() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "element")]
    struct Element {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        #[serde(rename = "custom-id")]
        id: u32,

        value: String,
    }

    let elem = Element {
        id: 999,
        value: "test".to_string(),
    };

    // Serialize to XML
    let xml = to_string(&elem).unwrap();
    assert_eq!(
        xml,
        r#"<element custom-id="999"><value>test</value></element>"#
    );

    // Deserialize from XML
    let deserialized: Element = from_str(&xml).unwrap();
    assert_eq!(deserialized, elem);
}

/// Test with text content and attributes
#[test]
fn test_text_content_with_attributes() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "message")]
    struct Message {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        lang: String,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        priority: u8,

        #[serde(rename = "$text")]
        content: String,
    }

    let msg = Message {
        lang: "en".to_string(),
        priority: 5,
        content: "Hello, World!".to_string(),
    };

    // Serialize to XML
    let xml = to_string(&msg).unwrap();
    assert_eq!(
        xml,
        r#"<message lang="en" priority="5">Hello, World!</message>"#
    );

    // Deserialize from XML
    let deserialized: Message = from_str(&xml).unwrap();
    assert_eq!(deserialized, msg);
}

/// Test roundtrip: serialize then deserialize should give same result
#[test]
fn test_roundtrip_symmetry() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "data")]
    struct Data {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        version: String,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        timestamp: i64,

        payload: String,
    }

    let original = Data {
        version: "2.0".to_string(),
        timestamp: 1234567890,
        payload: "test data".to_string(),
    };

    // Serialize
    let xml = to_string(&original).unwrap();

    // Deserialize
    let roundtrip: Data = from_str(&xml).unwrap();

    // Should be identical
    assert_eq!(roundtrip, original);
}

/// Test deeply nested structs with attributes at multiple levels
#[test]
fn test_deeply_nested_attributes() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "coordinates")]
    struct Coordinates {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        system: String,

        lat: f64,
        lon: f64,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "location")]
    struct Location {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        id: u32,

        name: String,
        coordinates: Coordinates,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "business")]
    struct Business {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        license: String,

        #[serde(with = "quick_xml::serde_helpers::attribute")]
        active: bool,

        title: String,
        location: Location,
    }

    let business = Business {
        license: "BIZ-123".to_string(),
        active: true,
        title: "Coffee Shop".to_string(),
        location: Location {
            id: 42,
            name: "Downtown".to_string(),
            coordinates: Coordinates {
                system: "WGS84".to_string(),
                lat: 40.7128,
                lon: -74.0060,
            },
        },
    };

    // Serialize to XML
    let xml = to_string(&business).unwrap();
    assert_eq!(
        xml,
        r#"<business license="BIZ-123" active="true"><title>Coffee Shop</title><location id="42"><name>Downtown</name><coordinates system="WGS84"><lat>40.7128</lat><lon>-74.006</lon></coordinates></location></business>"#
    );

    // Deserialize from XML
    let deserialized: Business = from_str(&xml).unwrap();
    assert_eq!(deserialized, business);
}

/// Test that field names are clean (no @ in JSON serialization would work)
#[test]
fn test_clean_field_names() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(rename = "record")]
    struct Record {
        #[serde(with = "quick_xml::serde_helpers::attribute")]
        id: u32,

        data: String,
    }

    let record = Record {
        id: 42,
        data: "content".to_string(),
    };

    // The XML should not have @ in attribute names
    let xml = to_string(&record).unwrap();
    assert!(!xml.contains('@'));
    assert_eq!(xml, r#"<record id="42"><data>content</data></record>"#);

    // The key point is that the field names in the struct are clean (no @)
    // This means the same struct can be used for JSON serialization
    // without getting @ in the JSON field names
}
