use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, hash::Hash};

// declarations
// ======================
// pub struct Symbol {
//     symbol: String,
//     data: String,
// }

// type Token = MyToken;

// ======================

// pub struct SymbolHandlers<T>(HashMap<String, Box<dyn Fn(Vec<T>) -> T>>);

pub trait Token: Clone {
    fn to_tree_node(&self) -> ParsingTreeNode;
}

pub struct ParsingTreeNode {
    pub symbol_type: String,
    pub data: String,
    pub children: Vec<ParsingTreeNode>,
}

impl ParsingTreeNode {
    pub fn build(symbol_type: String, data: String, children: Vec<ParsingTreeNode>) -> Self {
        ParsingTreeNode {
            symbol_type,
            data,
            children,
        }
    }
}

/// NodePair
/// a pair of a node and a state.
/// (TreeNode, state)
pub struct NodePair(ParsingTreeNode, usize);
impl NodePair {
    pub fn new(node: ParsingTreeNode, state: usize) -> Self {
        NodePair(node, state)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReduceDerivation {
    pub left: String,
    pub right: Vec<String>,
}

impl PartialEq for ReduceDerivation {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl Eq for ReduceDerivation {}

impl Hash for ReduceDerivation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.right.hash(state);
    }
}

impl ReduceDerivation {
    pub fn build(left: String, right: Vec<String>) -> Self {
        ReduceDerivation { left, right }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Shift(usize),
    Reduce(ReduceDerivation),
    Accept,
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub actions: HashMap<String, Action>,
}

impl State {
    pub fn new() -> Self {
        State {
            actions: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ActionTable {
    pub states: Vec<State>,
}

impl ActionTable {
    pub fn get_action(&self, state: usize, symbol: &str) -> Option<&Action> {
        self.states[state].actions.get(symbol)
    }
}

pub struct RParser {
    action_table: ActionTable,
    handlers: HashMap<ReduceDerivation, Box<dyn Fn(Vec<String>) -> String>>,
    // variables
    // ======================
    // ======================
}

impl RParser {
    pub const END_SYMBOL: &'static str = "__$__";
    pub const EPSILON_SYMBOL: &'static str = "__EPSILON__";
    pub const DUMMY_START_SYMBOL: &'static str = "__DUMMY_START__";

    pub fn new() -> Self {
        let action_table: ActionTable =  serde_json::from_str(r#"{"states":[{"actions":{"E":{"Shift":10},"__$__":"Accept","(":{"Shift":5},"int":{"Shift":3},"T":{"Shift":1}}},{"actions":{"__$__":{"Reduce":{"left":"E","right":["T"]}},"+":{"Shift":2},")":{"Reduce":{"left":"E","right":["T"]}}}},{"actions":{"T":{"Shift":1},"E":{"Shift":9},"(":{"Shift":5},"int":{"Shift":3}}},{"actions":{"+":{"Reduce":{"left":"T","right":["int"]}},")":{"Reduce":{"left":"T","right":["int"]}},"__$__":{"Reduce":{"left":"T","right":["int"]}},"*":{"Shift":4}}},{"actions":{"int":{"Shift":3},"(":{"Shift":5},"T":{"Shift":8}}},{"actions":{"E":{"Shift":6},"(":{"Shift":5},"int":{"Shift":3},"T":{"Shift":1}}},{"actions":{")":{"Shift":7}}},{"actions":{"__$__":{"Reduce":{"left":"T","right":["(","E",")"]}},")":{"Reduce":{"left":"T","right":["(","E",")"]}},"+":{"Reduce":{"left":"T","right":["(","E",")"]}}}},{"actions":{"__$__":{"Reduce":{"left":"T","right":["int","*","T"]}},"+":{"Reduce":{"left":"T","right":["int","*","T"]}},")":{"Reduce":{"left":"T","right":["int","*","T"]}}}},{"actions":{")":{"Reduce":{"left":"E","right":["T","+","E"]}},"__$__":{"Reduce":{"left":"E","right":["T","+","E"]}}}},{"actions":{"__$__":{"Reduce":{"left":"__DUMMY_START__","right":["E"]}}}}]}"#).unwrap();
        let mut handlers: HashMap<ReduceDerivation, Box<dyn Fn(Vec<String>) -> String>> =
            HashMap::new();

        handlers.insert(
            ReduceDerivation::build("T".into(), vec!["int".into()]),
            Box::new(|vals| {
                println!("reduce: T -> int");
                vals[0].clone()
            }),
        );
        handlers.insert(
            ReduceDerivation::build("T".into(), vec!["int".into(), "*".into(), "T".into()]),
            Box::new(|vals| {
                println!("reduce: T -> int * T");
                let left = vals[0].parse::<i64>().unwrap();
                let right = vals[2].parse::<i64>().unwrap();
                (left * right).to_string()
            }),
        );
        handlers.insert(
            ReduceDerivation::build("E".into(), vec!["T".into()]),
            Box::new(|vals| {
                println!("reduce: E -> T");
                vals[0].clone()
            }),
        );
        handlers.insert(
            ReduceDerivation::build(Self::DUMMY_START_SYMBOL.into(), vec!["E".into()]),
            Box::new(|vals| {
                println!("reduce: DUMMY_START -> E");
                vals[0].clone()
            }),
        );

        RParser {
            action_table,
            handlers,
        }
    }

    // do shift-reduce parsing
    pub fn parse<T>(&self, tokens: Vec<T>) -> Result<ParsingTreeNode, Box<dyn Error>>
    where
        T: Token,
    {
        let mut shift_index = 0;
        let mut stack: Vec<NodePair> = Vec::new();

        stack.push(NodePair::new(
            ParsingTreeNode::build(Self::DUMMY_START_SYMBOL.into(), String::new(), Vec::new()),
            0,
        ));

        loop {
            let token_node = &tokens[shift_index].to_tree_node();

            let action = self
                .action_table
                .get_action(stack.last().unwrap().1, &token_node.symbol_type);

            match action {
                Some(Action::Shift(next_state)) => {
                    // shift
                    stack.push(NodePair::new(
                        tokens[shift_index].to_tree_node(),
                        *next_state,
                    ));
                    shift_index += 1;
                }
                Some(Action::Reduce(derivation)) => {
                    // pop right hand
                    let mut children: Vec<ParsingTreeNode> = Vec::new();
                    let mut datas = Vec::new();
                    for _ in 0..derivation.right.len() {
                        if let Some(top) = stack.pop() {
                            datas.push(top.0.data.clone());
                            children.push(top.0);
                        } else {
                            panic!("error");
                        }
                    }

                    let children: Vec<_> = children.into_iter().rev().collect();
                    let datas: Vec<_> = datas.into_iter().rev().collect();
                    let handler = self.handlers.get(&derivation).unwrap();

                    // if the left hand side is dummy start symbol
                    // do nothing
                    if derivation.left == Self::DUMMY_START_SYMBOL {
                        stack.push(NodePair(
                            ParsingTreeNode::build(
                                derivation.left.to_string(),
                                handler(datas),
                                children,
                            ),
                            0,
                        ));
                        continue;
                    }

                    // goto[top_state(stack), X]
                    if let Action::Shift(next_state) = self
                        .action_table
                        .get_action(stack.last().unwrap().1, &derivation.left)
                        .unwrap()
                    {
                        stack.push(NodePair(
                            ParsingTreeNode::build(
                                derivation.left.to_string(),
                                handler(datas),
                                children,
                            ),
                            *next_state,
                        ));
                    } else {
                        panic!("error")
                    }
                }
                Some(Action::Accept) => {
                    let res = stack.pop().unwrap().0;
                    return Ok(res);
                }
                _ => {
                    println!("error");
                    return Err("error".into());
                }
            }
        }
    }
}
