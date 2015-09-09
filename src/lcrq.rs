/// Linked concurrent ring queue

use std::ptr;
use std::mem;
use std::marker::{Sync, Send};

use crq::CRQ;
use atomics::x86::compare_and_swap;

// `head` and `tail` are padded to get them on their very own cache lines.
// This assumes that usize is 64 bits, and a cache line is 64 bytes.
pub struct LCRQ {
    tail: *const CRQ,
    _pad_tail: [usize; 7],
    head: *const CRQ,
    _pad_head: [usize; 7],
}

unsafe impl Send for LCRQ {} // TODO: remove need for this
unsafe impl Sync for LCRQ {}

fn compare_and_swap_crq_ptr(destination: &*const CRQ, expected: *const CRQ, new_value: *const CRQ) -> bool {
    unsafe {
        compare_and_swap(mem::transmute(destination), mem::transmute(expected), mem::transmute(new_value))
    }
}

fn untracked_pointer(crq: CRQ) -> *const CRQ {
    unsafe { mem::transmute(Box::new(crq)) }
}

impl LCRQ {
    pub fn new() -> LCRQ {
        //let crq = untracked_crq();
        let crq = untracked_pointer(CRQ::new());
        LCRQ { tail: crq, head: crq, _pad_tail: [0; 7], _pad_head: [0; 7] }
    }

    pub fn dequeue(&self) -> Option<u64> {
        loop {
            let crq : &CRQ = unsafe { mem::transmute(self.head) };
            match crq.dequeue() {
                Some(value) => { return Some(value); }
                None => {
                    if (*crq).next == ptr::null() {
                        return None;
                    }
                    match crq.dequeue() {
                        Some(value) => { return Some(value); }
                        None => {
                            compare_and_swap_crq_ptr(&self.head, unsafe { mem::transmute(crq) }, crq.next);
                        }
                    }
                }
            }
        }
    }

    pub fn enqueue(&self, value: u64) {
        loop {
            let crq : &CRQ = unsafe { mem::transmute(self.tail) };

            if crq.next != ptr::null() {
                compare_and_swap_crq_ptr(&self.tail, crq, crq.next);
                continue;
            }

            match crq.enqueue(value) {
                Ok(_) => return,
                Err(_) => { // queue closed
                    let new_crq = CRQ::new();
                    new_crq.enqueue(value).ok().expect("Enqueue expected to always work on an empty queue");
                    let new_crq_ptr = untracked_pointer(new_crq);
                    if compare_and_swap_crq_ptr(&crq.next, ptr::null(), new_crq_ptr) {
                        compare_and_swap_crq_ptr(&self.tail, crq, new_crq_ptr);
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread::{ spawn, JoinHandle };
    use std::sync::Arc;
    use super::*;
    use crq::RING_SIZE;

    #[test]
    fn test_enqueue_ring_plus_one() {
        let lcrq = LCRQ::new();
        for i in 0..RING_SIZE+1 {
            lcrq.enqueue(100 + i as u64);
        }
    }

    #[test]
    fn test_enqueue_front_load() {
        let lcrq = LCRQ::new();
        for i in 0..RING_SIZE*10 {
            lcrq.enqueue(100 + i as u64);
        }
        for i in 0..RING_SIZE*10 {
            assert!(lcrq.dequeue() == Some(100 + i as u64));
        }
    }

    #[test]
    fn test_enqueue_and_deque_multithreaded() {
        let lcrq = Arc::new(LCRQ::new());

        let prod_lcrq = lcrq.clone();
        let cons_lcrq = lcrq.clone();

        let producer = spawn(move || {
            for i in 0..RING_SIZE*100 {
                prod_lcrq.enqueue(100 + i as u64);
            }
        });

        let consumer = spawn(move || {
            for i in 0..RING_SIZE*100 {
                loop {
                    match cons_lcrq.dequeue() {
                        Some(number) => { assert_eq!(number, 100 + i as u64); break },
                        None => { /* spin */ },
                    }
                }
            }
        });

        assert!(producer.join().is_ok());
        assert!(consumer.join().is_ok());
    }

    #[test]
    fn multi_producer_single_consumer() {
        let lcrq = Arc::new(LCRQ::new());

        let producer_1 = start_producer(lcrq.clone(), 100000, 100100);
        let producer_2 = start_producer(lcrq.clone(), 100100, 100200);

        let cons_lcrq = lcrq.clone();
        let consumer = spawn(move || {
            for _ in 0..200 {
                loop {
                    match cons_lcrq.dequeue() {
                        Some(number) => { assert!(number >= 100000); assert!(number < 100200); break },
                        None => { /* spin */ },
                    }
                }
            }
        });

        assert!(producer_1.join().is_ok());
        assert!(producer_2.join().is_ok());
        assert!(consumer.join().is_ok());
    }

    fn start_producer(queue: Arc<LCRQ>, start: u64, end: u64) -> JoinHandle<()> {
        spawn(move || {
            for i in start..end {
                queue.enqueue(i);
            }
        })
    }

}
