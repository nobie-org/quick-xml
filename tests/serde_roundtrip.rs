use quick_xml::{de::from_str, se::to_string};
use serde::{Deserialize, Serialize};

use pretty_assertions::assert_eq;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Node {
    Boolean(bool),
    Identifier { value: String, index: u32 },
    EOF,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Nodes {
    #[serde(rename = "$value")]
    items: Vec<Node>,
}

#[test]
#[ignore]
fn round_trip_list_of_enums() {
    // Construct some inputs
    let nodes = Nodes {
        items: vec![
            Node::Boolean(true),
            Node::Identifier {
                value: "foo".to_string(),
                index: 5,
            },
            Node::EOF,
        ],
    };

    let should_be = r#"
    <Nodes>
        <Boolean>
            true
        </Boolean>
        <Identifier>
            <value>foo</value>
            <index>5</index>
        </Identifier>
        <EOF />
    </Nodes>"#;

    let serialized_nodes = to_string(&nodes).unwrap();
    assert_eq!(serialized_nodes, should_be);

    // Then turn it back into a `Nodes` struct and make sure it's the same
    // as the original
    let deserialized_nodes: Nodes = from_str(serialized_nodes.as_str()).unwrap();
    assert_eq!(deserialized_nodes, nodes);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum RoundE {
    Unit,
    Newtype(bool),
    Tuple(f64, String),
    Struct { float: f64, string: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct RoundRoot {
    field: RoundE,
}

#[test]
fn roundtrip_enum_in_field_newtype() {
    let v = RoundRoot {
        field: RoundE::Newtype(true),
    };
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<field><Newtype>true</Newtype></field>"));
    let back: RoundRoot = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_enum_in_field_unit() {
    let v = RoundRoot {
        field: RoundE::Unit,
    };
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<field>Unit</field>"));
    let back: RoundRoot = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

// Nested enums: enum containing another enum as newtype variant
#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum InnerE {
    Unit,
    Newtype(bool),
    Tuple(f64, String),
    Struct { float: f64, string: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum OuterWrapE {
    Unit,
    Wrap(InnerE),
}

#[test]
fn roundtrip_enum_in_enum_unit() {
    let v = OuterWrapE::Wrap(InnerE::Unit);
    let xml = to_string(&v).unwrap();
    // Unit inner enum inside newtype variant serializes as text content of <Wrap>
    assert!(xml.contains("<Wrap>Unit</Wrap>"));
    let back: OuterWrapE = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_enum_in_enum_newtype() {
    let v = OuterWrapE::Wrap(InnerE::Newtype(true));
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<Wrap><Newtype>true</Newtype></Wrap>"));
    let back: OuterWrapE = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_enum_in_enum_tuple() {
    let v = OuterWrapE::Wrap(InnerE::Tuple(42.0, "answer".into()));
    let xml = to_string(&v).unwrap();
    assert!(
        xml.contains("<Wrap><Tuple>42</Tuple><Tuple>answer</Tuple></Wrap>")
            || xml.contains("<Wrap><Tuple>42</Tuple>\n  <Tuple>answer</Tuple></Wrap>")
    );
    let back: OuterWrapE = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_enum_in_enum_struct() {
    let v = OuterWrapE::Wrap(InnerE::Struct {
        float: 42.0,
        string: "answer".into(),
    });
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<Wrap><Struct><float>42</float><string>answer</string></Struct></Wrap>"));
    let back: OuterWrapE = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

// Enum embedded in another enum's struct variant
#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum OuterStructE {
    Unit,
    StructWrap { inner: InnerE },
}

#[test]
fn roundtrip_enum_in_enum_struct_variant_unit() {
    let v = OuterStructE::StructWrap {
        inner: InnerE::Unit,
    };
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<StructWrap><inner>Unit</inner></StructWrap>"));
    let back: OuterStructE = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_enum_in_enum_struct_variant_struct() {
    let v = OuterStructE::StructWrap {
        inner: InnerE::Struct {
            float: 42.0,
            string: "answer".into(),
        },
    };
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<StructWrap><inner><Struct><float>42</float><string>answer</string></Struct></inner></StructWrap>"));
    let back: OuterStructE = from_str(&xml).unwrap();
    assert_eq!(back, v);
}
#[test]
fn roundtrip_enum_in_field_tuple() {
    let v = RoundRoot {
        field: RoundE::Tuple(42.0, "answer".into()),
    };
    let xml = to_string(&v).unwrap();
    assert!(xml.contains("<field><Tuple>42</Tuple><Tuple>answer</Tuple></field>"));
    let back: RoundRoot = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_enum_in_field_struct() {
    let v = RoundRoot {
        field: RoundE::Struct {
            float: 42.0,
            string: "answer".into(),
        },
    };
    let xml = to_string(&v).unwrap();
    assert!(
        xml.contains("<field><Struct><float>42</float><string>answer</string></Struct></field>")
    );
    let back: RoundRoot = from_str(&xml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn roundtrip_from_xml_enum_in_field() {
    // Ensure XML -> struct -> XML is stable for newtype
    let xml = "<RoundRoot><field><Newtype>true</Newtype></field></RoundRoot>";
    let v: RoundRoot = from_str(xml).unwrap();
    assert_eq!(
        v,
        RoundRoot {
            field: RoundE::Newtype(true)
        }
    );
    let xml2 = to_string(&v).unwrap();
    assert!(xml2.contains("<field><Newtype>true</Newtype></field>"));
}
