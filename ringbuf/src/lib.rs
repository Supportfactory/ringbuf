// Copyright 2021 - SupportFactory.net
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::collections::VecDeque;

pub struct RingBuffer {
    queue: VecDeque<u8>,
    // `VecDeque` doesn't use the exact capacity we pass to it, so we need this field.
    capacity: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        RingBuffer {
            // There is no `with_exact_capacity`, and `reserve_exact` just calls `reserve`,
            // so there's no point in trying to fight the excess capacity.
            queue: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn peek(&mut self, n: usize) -> &[u8] {
        if n > self.queue.len() {
            panic!("oob");
        }

        let need_contiguous = {
            let (left, _) = self.queue.as_slices();
            n > left.len()
        };

        if need_contiguous {
            self.queue.make_contiguous();
        }

        let (left, _) = self.queue.as_slices();
        &left[..n]
    }

    fn skip(&mut self, n: usize) {
        if n > self.queue.len() {
            panic!("oob");
        }

        self.queue.drain(..n);
    }

    fn push(&mut self, bytes: &[u8]) {
        let leeway = self.capacity - self.queue.len();

        if bytes.len() <= leeway {
            // There's enough leeway to insert all bytes.
            self.queue.extend(bytes);
        } else if bytes.len() >= self.capacity {
            // Not enough room to fit everything, drop all contents and extend from the tail.
            self.queue.clear();
            self.queue.extend(&bytes[bytes.len() - self.capacity..]);
        } else {
            // Make enough room to fit everything.
            self.queue.drain(..bytes.len() - leeway);
            self.queue.extend(bytes);
        }
    }
}

/// Creates a new ring buffer of the specified capacity.
#[no_mangle]
pub extern "C" fn new(capacity: usize) -> *mut RingBuffer {
    Box::into_raw(Box::new(RingBuffer::new(capacity)))
}

/// How much data can be read from the buffer?
///
/// It is undefined behaviour to pass a pointer not pointing to a non-deleted `RingBuffer`.
#[no_mangle]
pub extern "C" fn read_available(buffer: *mut RingBuffer) -> usize {
    let buffer = unsafe { &mut *buffer };
    buffer.queue.len()
}

/// How much data can be written into the buffer without overwriting contents?
///
/// It is undefined behaviour to pass a pointer not pointing to a non-deleted `RingBuffer`.
#[no_mangle]
pub extern "C" fn write_available(buffer: *mut RingBuffer) -> usize {
    let buffer = unsafe { &mut *buffer };
    buffer.capacity - buffer.queue.len()
}

/// Peeks from the buffer.
///
/// Panics if one tries to read more than available in the buffer.
///
/// The results should **not** be read from after pushing or deleting the buffer.
///
/// It is undefined behaviour to pass a pointer not pointing to a non-deleted `RingBuffer`.
#[no_mangle]
pub extern "C" fn peek(buffer: *mut RingBuffer, n: usize) -> *const u8 {
    let buffer = unsafe { &mut *buffer };
    buffer.peek(n).as_ptr()
}

/// Skips data from the buffer.
///
/// Panics if one tries to skip more than available in the buffer.
///
/// It is undefined behaviour to pass a pointer not pointing to a non-deleted `RingBuffer`.
#[no_mangle]
pub extern "C" fn skip(buffer: *mut RingBuffer, n: usize) {
    let buffer = unsafe { &mut *buffer };
    buffer.skip(n)
}

/// Pushes data to the buffer.
///
/// It is undefined behaviour to pass a pointer not pointing to a non-deleted `RingBuffer`,
/// or to pass an invalid pointer to bytes which is not of the matching length.
#[no_mangle]
pub extern "C" fn push(buffer: *mut RingBuffer, bytes: *const u8, n: usize) {
    let buffer = unsafe { &mut *buffer };
    let bytes = unsafe { std::slice::from_raw_parts(bytes, n) };
    buffer.push(bytes)
}

/// It is undefined behaviour to pass a pointer not pointing to a non-deleted `RingBuffer`.
#[no_mangle]
pub extern "C" fn del(buffer: *mut RingBuffer) {
    let buffer = unsafe { Box::from_raw(buffer) };
    drop(buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_push_paths() {
        let mut buffer = RingBuffer::new(4);

        // Enough room.
        buffer.push(&[1, 2, 3]);
        assert_eq!(buffer.queue.len(), 3);
        assert_eq!(buffer.queue, &[1, 2, 3]);

        // Not enough room.
        buffer.push(&[1, 2, 3]);
        assert_eq!(buffer.queue.len(), 4);
        assert_eq!(buffer.queue, &[3, 1, 2, 3]);

        // Not enough room or capacity.
        buffer.push(&[1, 2, 3, 4, 5]);
        assert_eq!(buffer.queue.len(), 4);
        assert_eq!(buffer.queue, &[2, 3, 4, 5]);
    }
}
