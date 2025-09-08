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
