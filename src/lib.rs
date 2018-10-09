extern crate sxd_document;

use std::str;
use std::string::String;
use std::collections::HashMap;

use sxd_document::{
    parser,
    Package,
};
use sxd_document::dom::{
    Document,
    Root,
    ChildOfRoot,
    Element,
    ChildOfElement,
};

#[derive(Debug)]
pub struct Program {
    pub groups: Vec<StatementBody>
}

#[derive(PartialEq, Debug)]
pub struct StatementBody {
    pub blocks: Vec<Block>
}

#[derive(PartialEq, Debug)]
pub struct Block {
    pub block_type: String,
    pub id: String,
    pub fields: Option<HashMap<String, FieldValue>>,
    pub statements: Option<HashMap<String, StatementBody>>,
}

#[derive(PartialEq, Debug)]
pub enum FieldValue {
    SimpleField(String),
    ExpressionField(Block),
}

impl Program {
    pub fn new() -> Self {
        Self {
            groups: Vec::new()
        }
    }
}

impl StatementBody {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new()
        }
    }
}

impl Block {
    pub fn new() -> Self {
        Self {
            block_type: "".to_string(),
            id: "".to_string(),
            fields: None,
            statements: None
        }
    }

    pub fn add_field(&mut self, name: String, value: FieldValue) {
        match self.fields {
            Some(ref mut f) => {
                f.insert(name.to_owned(), value);
            },
            None => {
                let mut new_map: HashMap<String, FieldValue> = HashMap::new();
                new_map.insert(name.to_owned(), value);
                self.fields = Some(new_map);
            }
        }
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldValue> {
        match self.fields {
            Some(ref fields) => {
                fields.get(name)
            },
            None => None
        }
    }

    pub fn add_statement_body(&mut self, name: String, body: StatementBody) {
        match self.statements {
            Some(ref mut s) => {
                s.insert(name.to_owned(), body);
            },
            None => {
                let mut new_map: HashMap<String, StatementBody> = HashMap::new();
                new_map.insert(name.to_owned(), body);
                self.statements = Some(new_map);
            }
        };
    }

    pub fn get_statement(&self, name: &str) -> Option<&StatementBody> {
        match self.statements {
            Some(ref statements) => {
                statements.get(name)
            },
            None => None
        }
    }
}


// Utilities for creating Blockly data structures

pub fn program_from_xml(xml: &str) -> Program {
    let mut program = Program::new();

    let package: Package = parser::parse(xml).expect("Failed to parse XML!");
    let document: Document = package.as_document();

    let xml_element = get_xml_element(document);

    for child in xml_element.children().iter() {
        match child {
            &ChildOfElement::Element(el) => {
                let element_name = el.name().local_part();
                match element_name {
                    "block" => {
                        program.groups.push(make_statement_body(el));
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    program
}

fn get_next_block_element(block_el: Element) -> Option<Element> {
    let next_el: Option<Element> = {
        let mut found: Option<Element> = None;
        for child in block_el.children().iter() {
            match child {
                &ChildOfElement::Element(el) => {
                    match el.name().local_part() {
                        "next" => {
                            found = Some(el);
                            break;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        found
    };

    match next_el {
        Some(next_el) => {
            for child in next_el.children().iter() {
                match child {
                    &ChildOfElement::Element(el) => {
                        match el.name().local_part() {
                            "block" => {
                                return Some(el);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
            None
        },
        None => None
    }
}

fn make_statement_body(first_block: Element) -> StatementBody {
    let mut body = StatementBody::new();

    // Create each block, put them into the statement body
    let mut block_el: Element;
    block_el = first_block;
    loop {
        body.blocks.push(make_block(block_el));
        let next_block = get_next_block_element(block_el);
        match next_block {
            Some(next_block_el) => {
                block_el = next_block_el;
            },
            None => {
                break;
            }
        }
    }

    body
}

fn make_block(block_el: Element) -> Block {
    let mut block = Block::new();
    for attribute in block_el.attributes().iter() {
        let name = attribute.name().local_part();
        let value = attribute.value().to_string();
        match name {
            "type" => { block.block_type = value; },
            "id" => { block.id = value; },
            _ => {}
        }
    }
    for child in block_el.children().iter() {
        match child {
            &ChildOfElement::Element(child_el) => {
                let child_name = child_el.name().local_part();
                match child_name {
                    "statement" => {
                        let statement_el = child_el;
                        let statement_name = get_attribute(statement_el, "name").unwrap();
                        let statement_body = match get_first_child_element(statement_el) {
                            Some(first_child_block) => make_statement_body(first_child_block),
                            None => StatementBody::new()
                        };
                        block.add_statement_body(statement_name, statement_body);
                    },
                    "field" => {
                        let field_el = child_el;
                        let field_name = get_attribute(field_el, "name").unwrap();
                        let field_value = make_field_value(field_el);
                        block.add_field(field_name, field_value);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
    block
}

fn make_field_value(field_el: Element) -> FieldValue {
    for child in field_el.children().iter() {
        match child {
            &ChildOfElement::Text(text_node) => {
                let value = text_node.text().to_string();
                return FieldValue::SimpleField(value);
            },
            _ => panic!("TODO: Implement expression fields")
        }
    }
    panic!("Expected child nodes for field");
}

// General DOM utilities

fn get_xml_element(document: Document) -> Element {
    let root: Root = document.root();
    let root_children = root.children();
    for child in root_children.iter() {
        match child {
            &ChildOfRoot::Element(el) => {
                let element_name = el.name().local_part();
                if element_name == "xml" {
                    return el;
                }
            },
            _ => {}
        }
    }
    panic!("Cannot find xml element!");
}

fn get_first_child_element(element: Element) -> Option<Element> {
    for child in element.children().iter() {
        match child {
            &ChildOfElement::Element(el) => {
                return Some(el);
            },
            _ => {}
        }
    }
    None
}

fn get_attribute(element: Element, attribute_name: &str) -> Option<String> {
    for attribute in element.attributes().iter() {
        let name = attribute.name().local_part();
        let value = attribute.value().to_string();
        if name == attribute_name {
            return Some(value);
        }
    }
    None
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_block() {
        let xml: &str = r#"
            <block type="inner_loop" id="]Lb|t?wfd#;s)[llJx8Y">
                <field name="COUNT">3</field>
                <statement name="BODY">
                </statement>
            </block>
        "#;

        // TODO: move into test helper function
        let package: Package = parser::parse(xml).expect("Failed to parse XML!");
        let document: Document = package.as_document();
        let mut root_element: Option<Element> = None;
        for child in document.root().children().iter() {
            match child {
                &ChildOfRoot::Element(el) => {
                    root_element = Some(el);
                    break;
                },
                _ => {}
            }
        }

        assert!(root_element.is_some());
        let block = make_block(root_element.unwrap());
        assert_eq!(block.block_type, "inner_loop");
        assert_eq!(block.id, "]Lb|t?wfd#;s)[llJx8Y");
        assert!(block.fields.is_some());
        assert!(block.statements.is_some());
        let count_field = block.get_field("COUNT");
        assert!(count_field.is_some());
        assert_eq!(count_field.unwrap(), &FieldValue::SimpleField("3".to_string()));
    }

    #[test]
    fn test_get_next_block_element() {
        let xml: &str = r#"
            <block type="led_on" id="^3xb.m4E9i0;3$R10(=5">
                <field name="TIME">300</field>
                <next>
                    <block type="led_off" id="HX4*sB9=gbJtq$Y{ke6b">
                        <field name="TIME">100</field>
                    </block>
                </next>
            </block>
        "#;

        // TODO: move into test helper function
        let package: Package = parser::parse(xml).expect("Failed to parse XML!");
        let document: Document = package.as_document();
        let mut root_element: Option<Element> = None;
        for child in document.root().children().iter() {
            match child {
                &ChildOfRoot::Element(el) => {
                    root_element = Some(el);
                    break;
                },
                _ => {}
            }
        }

        assert!(root_element.is_some());
        let next_block = get_next_block_element(root_element.unwrap());
        assert!(next_block.is_some());
        let next_block_unwrapped = next_block.unwrap();
        assert_eq!(get_attribute(next_block_unwrapped, "type"), Some("led_off".to_string()));
        assert_eq!(get_attribute(next_block_unwrapped, "id"), Some("HX4*sB9=gbJtq$Y{ke6b".to_string()));
    }

    #[test]
    fn test_program_from_xml_advanced() {
        let xml: &str = r#"
            <xml xmlns="http://www.w3.org/1999/xhtml">
                <variables></variables>
                <block type="main_loop" id="[.)/fqUYv92(mzb{?:~u" deletable="false" movable="false" x="50" y="50">
                    <statement name="BODY">
                        <block type="inner_loop" id="]Lb|t?wfd#;s)[llJx8Y">
                            <field name="COUNT">3</field>
                            <statement name="BODY">
                                <block type="led_on" id="^3xb.m4E9i0;3$R10(=5">
                                    <field name="TIME">300</field>
                                    <next>
                                        <block type="led_off" id="HX4*sB9=gbJtq$Y{ke6b">
                                            <field name="TIME">100</field>
                                        </block>
                                    </next>
                                </block>
                            </statement>
                            <next>
                                <block type="led_on" id="kB~f~7W`wkGa0i4z3mHw">
                                    <field name="TIME">100</field>
                                    <next>
                                        <block type="led_off" id="$fdlZB)btzA8YtB/!xz`">
                                            <field name="TIME">100</field>
                                        </block>
                                    </next>
                                </block>
                            </next>
                        </block>
                    </statement>
                </block>
            </xml>
        "#;

        let program: Program = program_from_xml(xml);
        assert_eq!(program.groups.len(), 1);

        let group = program.groups.get(0).unwrap();
        assert_eq!(group.blocks.len(), 1);

        let main_loop_block = group.blocks.get(0).unwrap();
        assert_eq!(main_loop_block.block_type, "main_loop");
        assert_eq!(main_loop_block.id, "[.)/fqUYv92(mzb{?:~u");
        assert!(main_loop_block.statements.is_some());

        let main_loop_statements = main_loop_block.statements.as_ref().unwrap();
        assert_eq!(main_loop_statements.len(), 1);
        assert!(main_loop_statements.contains_key("BODY"));

        let main_loop_body = main_loop_statements.get("BODY");
        let main_loop_body_statement = main_loop_body.as_ref().unwrap();
        assert_eq!(main_loop_body_statement.blocks.len(), 3);

        let inner_loop_block = main_loop_body_statement.blocks.get(0).unwrap();
        assert_eq!(inner_loop_block.block_type, "inner_loop");
        assert_eq!(inner_loop_block.id, "]Lb|t?wfd#;s)[llJx8Y");
        assert!(inner_loop_block.fields.is_some());
        assert_eq!(inner_loop_block.get_field("COUNT"), Some(&FieldValue::SimpleField("3".to_string())));

        let inner_loop_statement_maybe = inner_loop_block.get_statement("BODY");
        assert!(inner_loop_statement_maybe.is_some());
        let inner_loop_statement = inner_loop_statement_maybe.unwrap();
        assert_eq!(inner_loop_statement.blocks.len(), 2);

        let led_on_block = inner_loop_statement.blocks.get(0).unwrap();
        assert_eq!(led_on_block.block_type, "led_on");
        assert_eq!(led_on_block.id, "^3xb.m4E9i0;3$R10(=5");
        assert_eq!(led_on_block.get_field("TIME"), Some(&FieldValue::SimpleField("300".to_string())));

        let led_off_block = inner_loop_statement.blocks.get(1).unwrap();
        assert_eq!(led_off_block.block_type, "led_off");
        assert_eq!(led_off_block.id, "HX4*sB9=gbJtq$Y{ke6b");
    }
}