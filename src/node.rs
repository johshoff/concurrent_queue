use flag_and_u63::FlagAndU63;

// TODO: abstract away
pub const NODE_VALUE_EMPTY: u64 = 0;

pub struct Node {
    // TODO: The layout of these needs to be in the following order, without padding. (See use of compare_and_swap_2)
    index_and_safe: FlagAndU63, // highest bit: safe, remaining 63 bits: value
    value: u64,
    // TODO: pad to cache line size... Assume L2 cache?
}

impl Node {
    pub fn new(index: u64, value: u64, safe: bool) -> Node {
        Node { index_and_safe: FlagAndU63::new(safe, index), value: value }
    }

    pub fn is_safe(&self) -> bool {
        self.index_and_safe.is_flag_set()
    }

    pub fn index(&self) -> u64 {
        self.index_and_safe.value()
    }

    pub fn safe_and_index(&self) -> (bool, u64) {
        self.index_and_safe.flag_and_value()
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn set_safe(&mut self) {
        self.index_and_safe.set_flag();
    }

    pub fn set_unsafe(&mut self) {
        self.index_and_safe.unset_flag();
    }
}


#[test]
fn test_node_value() {
    assert_eq!(Node::new(0, 1, true).value(), 1);
    assert_eq!(Node::new(5, 9, true).value(), 9);
    assert_eq!(Node::new(8, 2, true).value(), 2);
    assert_eq!(Node::new(0, 1, false).value(), 1);
    assert_eq!(Node::new(5, 9, false).value(), 9);
    assert_eq!(Node::new(8, 2, false).value(), 2);
}

#[test]
fn test_node_index() {
    assert_eq!(Node::new(0, 1, true).index(), 0);
    assert_eq!(Node::new(5, 9, true).index(), 5);
    assert_eq!(Node::new(8, 2, true).index(), 8);
    assert_eq!(Node::new(0, 1, false).index(), 0);
    assert_eq!(Node::new(5, 9, false).index(), 5);
    assert_eq!(Node::new(8, 2, false).index(), 8);
}

#[test]
fn test_node_safe() {
    let node = Node::new(0, 0, true);
    assert!(node.is_safe());

    let node = Node::new(0, 0, false);
    assert!(!node.is_safe());

    let mut node = Node::new(1, 2, true);
    assert!(node.is_safe());
    assert_eq!(node.index(), 1);
    assert_eq!(node.value(), 2);

    node.set_unsafe();
    assert!(!node.is_safe());
    assert_eq!(node.index(), 1);
    assert_eq!(node.value(), 2);

    node.set_safe();
    assert!(node.is_safe());
    assert_eq!(node.index(), 1);
    assert_eq!(node.value(), 2);
}

