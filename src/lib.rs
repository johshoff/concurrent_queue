#![feature(asm)]
#![feature(repr_simd)]

use std::ptr;
use std::mem;
use std::marker::{Sync, Send};

pub mod flag_and_u63; // TODO: Using `pub` only to suppress unused warnings
use flag_and_u63::FlagAndU63;

pub mod node; // TODO: Using `pub` only to suppress unused warnings
use node::{ Node, NODE_VALUE_EMPTY };

mod atomics;
use atomics::x86::*;

fn compare_and_swap_nodes(node: &Node, expected: &Node, new_value: &Node) -> bool {
    let mem_current   : &DoubleU64 = unsafe { mem::transmute(node)      };
    let mem_expected  : &DoubleU64 = unsafe { mem::transmute(expected)  };
    let mem_new_value : &DoubleU64 = unsafe { mem::transmute(new_value) };

    compare_and_swap_2(mem_current, mem_expected, mem_new_value)
}

pub const RING_SIZE: usize = 4;

pub struct CRQ { // TODO: ensure fields are on distinct cache lines
    head: u64,   // read location
    tail_and_closed: FlagAndU63, // flag: queue closed, u63: tail (write location)
    next: *mut CRQ,
    ring: [Node; RING_SIZE]
}

unsafe impl Send for CRQ {} // TODO: remove need for this
unsafe impl Sync for CRQ {}

pub struct QueueClosed;

impl CRQ {
    pub fn new() -> CRQ {
        /*
            TODO: It would be nice to do this without unsafe. I can create a vector:

                let ring = (0..RING_SIZE).map(|u| Node::new(u, NODE_VALUE_EMPTY)).collect::<Vec<Node>>();

            But how do I get the underlying array? Getting a slice `&ring[..]` gets me a [Node] which is almost there, but it's unsized.
        */
        let ring = unsafe {
            let mut ring: [Node; RING_SIZE] = mem::uninitialized();

            for (i, element) in ring.iter_mut().enumerate() {
                let value = Node::new(i as u64, NODE_VALUE_EMPTY, true);
                ptr::write(element, value);
            }

            ring
        };

        CRQ { head: 0, tail_and_closed: FlagAndU63::new(false, 0), next: ptr::null_mut(), ring: ring }
    }

    pub fn enqueue(&self, new_value: u64) -> Result<(), QueueClosed> {
        loop {
            let current_tail_and_closed = FlagAndU63::from_repr(fetch_and_add(self.tail_and_closed.ref_combined(), 1));
            let (closed, tail) = current_tail_and_closed.flag_and_value();

            if closed {
                return Err(QueueClosed);
            }

            {
                let node = &self.ring[tail as usize % RING_SIZE]; // TODO: are we doing a range check? not needed
                let value = node.value();

                if value == NODE_VALUE_EMPTY {
                    let (is_safe, index) = node.safe_and_index();
                    if index <= tail &&
                       (is_safe || self.head <= tail) &&
                       compare_and_swap_nodes(node, &Node::new(index, NODE_VALUE_EMPTY, is_safe), &Node::new(tail, new_value, true)) {
                        return Ok(());
                    }
                }
            }

            // NOTE: Checking `head < tail` is necessary to avoid underflow in `tail - head`, since
            // head can advance beyond tail
            let head = self.head;
            if (head < tail && (tail - head) as usize >= RING_SIZE) || self.is_starving() {
                test_and_set(self.tail_and_closed.ref_combined());
                return Err(QueueClosed);
            }

        }
    }

    pub fn dequeue(&self) -> Option<u64> {
        loop {
            let head = fetch_and_add(&self.head, 1);
            {
                let node = &self.ring[head as usize % RING_SIZE]; // TODO: are we doing a range check? not needed

                loop {
                    let value = node.value();
                    let (is_safe, index) = node.safe_and_index();

                    if index > head {
                        break;
                    }

                    if value != NODE_VALUE_EMPTY {
                        if index == head {
                            if compare_and_swap_nodes(node, &Node::new(head, value, is_safe), &Node::new(head + RING_SIZE as u64, NODE_VALUE_EMPTY, is_safe)) {
                                return Some(value)
                            }
                        } else {
                            if compare_and_swap_nodes(node, &Node::new(index, value, is_safe), &Node::new(index, value, false)) {
                                break;
                            }
                        }
                    } else {
                        if compare_and_swap_nodes(node, &Node::new(index, NODE_VALUE_EMPTY, is_safe), &Node::new(head + RING_SIZE as u64, NODE_VALUE_EMPTY, is_safe)) {
                            break;
                        }
                    }
                }
            }
            let tail = self.tail_and_closed.value();
            if tail <= head + 1 {
                self.fix_state();
                return None;
            }

        }
    }

    fn is_starving(&self) -> bool {
        // TODO: IMPLEMENT
        false
    }

    fn fix_state(&self) {
        loop {
            let tail_repr = fetch_and_add(self.tail_and_closed.ref_combined(), 0);
            let head = fetch_and_add(&self.head, 0);

            if self.tail_and_closed.combined() != tail_repr {
                continue;
            }

            if head <= tail_repr {
                return; // nothing to do
            }

            // jh: Since tail_repr < head at this point it means that tail_repr does not have a it's highest bit set (the CLOSED bit).
            //     Alternatively, it means that head has the highest bit set, and I guess that'll just close the queue?

            if compare_and_swap(self.tail_and_closed.ref_combined(), tail_repr, head) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::ptr;
    use std::thread::spawn;
    use super::*;
    use node::NODE_VALUE_EMPTY;

    #[test]
    fn new_crq() {
        let crq = CRQ::new();
        assert_eq!(crq.head, 0);
        assert_eq!(crq.tail_and_closed.value(), 0);
        assert!(!crq.tail_and_closed.is_flag_set());
        assert_eq!(crq.next, ptr::null_mut());
        assert_eq!(crq.ring.len(), RING_SIZE);

        for (i, element) in crq.ring.iter().enumerate() {
            assert!(element.is_safe());
            assert_eq!(element.index(), i as u64);
            assert_eq!(element.value(), NODE_VALUE_EMPTY);
        }
    }

    #[test]
    fn test_full_queue() {
        let crq = CRQ::new();
        for _ in 0..RING_SIZE {
            assert!(crq.enqueue(100).is_ok());
        }
        assert!(crq.enqueue(100).is_err());
    }

    #[test]
    fn test_deque_empty() {
        let crq = CRQ::new();
        assert!(crq.dequeue() == None);
    }

    #[test]
    fn test_enqueue_and_deque() {
        let crq = CRQ::new();
        for i in 0..RING_SIZE {
            assert!(crq.enqueue(100 + i as u64).is_ok());
        }

        for i in 0..RING_SIZE {
            assert!(crq.dequeue() == Some(100 + i as u64));
        }
    }

    #[test]
    fn test_enqueue_and_deque_laps() {
        let crq = CRQ::new();
        for i in 0..RING_SIZE*10 {
            assert!(crq.enqueue(100 + i as u64).is_ok());
            assert!(crq.dequeue() == Some(100 + i as u64));
        }
    }

    #[test]
    fn test_enqueue_and_deque_multithreaded() {
        let crq = Arc::new(CRQ::new());

        let prod_crq = crq.clone();
        let cons_crq = crq.clone();

        let producer = spawn(move || {
            for i in 0..RING_SIZE {
                loop {
                    match prod_crq.enqueue(100 + i as u64) {
                        Ok(()) => { break; },
                        Err(QueueClosed) => { panic!("Queue closed"); },
                    }
                }
            }
        });

        let consumer = spawn(move || {
            for i in 0..RING_SIZE {
                loop {
                    match cons_crq.dequeue() {
                        Some(number) => { assert_eq!(number, 100 + i as u64); break },
                        None => { /* spin */ },
                    }
                }
            }
        });

        assert!(producer.join().is_ok());
        assert!(consumer.join().is_ok());
    }
}
