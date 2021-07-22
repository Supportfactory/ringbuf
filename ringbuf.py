# Copyright 2021 - SupportFactory.net
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
import ctypes
import ctypes.util
import functools
import threading

_rb = ctypes.cdll.LoadLibrary(ctypes.util.find_library("ringbuf"))

_rb_new = _rb.new
_rb_new.argtypes = (ctypes.c_size_t,)
_rb_new.restype = ctypes.c_void_p

_rb_read_available = _rb.read_available
_rb_read_available.argtypes = (ctypes.c_void_p,)
_rb_read_available.restype = ctypes.c_size_t

_rb_write_available = _rb.write_available
_rb_write_available.argtypes = (ctypes.c_void_p,)
_rb_write_available.restype = ctypes.c_size_t

_rb_peek = _rb.peek
_rb_peek.argtypes = (ctypes.c_void_p, ctypes.c_size_t,)
_rb_peek.restype = ctypes.POINTER(ctypes.c_uint8)

_rb_skip = _rb.skip
_rb_skip.argtypes = (ctypes.c_void_p, ctypes.c_size_t,)

_rb_push = _rb.push
_rb_push.argtypes = (ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,)

_rb_del = getattr(_rb, 'del')
_rb_del.argtypes = (ctypes.c_void_p,)

def _check_thread(f):
    @functools.wraps(f)
    def wrapped(self, *args, **kwargs):
        if threading.get_ident() != self._RingBuffer__tid:
            raise RuntimeError('RingBuffer is not thread-safe, but was used from a different thread')

        return f(self, *args, **kwargs)

    return wrapped

class RingBuffer:
    """
    A memory-wise efficient Ring Buffer implementation for working with `bytes`.
    """
    def __init__(self, capacity: int):
        """
        Create a new Ring Buffer instance with the given fixed capacity.

        The Ring Buffer will **not** grow when attempting to write past its capacity.
        """
        self.__buffer = _rb_new(capacity)
        self.__tid = threading.get_ident()

    @property
    @_check_thread
    def read_available(self):
        """
        Return the number of bytes which can be read from the queue.
        """
        return _rb_read_available(self.__buffer)

    @property
    @_check_thread
    def write_available(self):
        """
        Return the number of bytes that can be written into the queue.
        """
        return _rb_write_available(self.__buffer)

    @_check_thread
    def peek(self, n):
        """
        Peek `n` bytes from the buffer, without removing them from the queue.

        Attempting to read more than `read_available` bytes will raise an error.
        """
        if n > self.read_available:
            raise ValueError(f'Cannot read more than {self.read_available}')

        buffer = ctypes.create_string_buffer(n)
        ptr = _rb_peek(self.__buffer, n)
        ctypes.memmove(buffer, ptr, n)
        return buffer.raw

    @_check_thread
    def skip(self, n):
        """
        Skip `n` bytes from the buffer.

        Attempting to skip more than `read_available` bytes will raise an error.
        """
        if n > self.read_available:
            raise ValueError(f'Cannot skip more than {self.read_available}')

        _rb_skip(self.__buffer, n)

    @_check_thread
    def push(self, data: bytes):
        """
        Push the given data bytes to the end of the buffer.

        Attempting to push more than `write_available` will overwrite the oldest bytes.
        """
        buffer = ctypes.create_string_buffer(data)
        _rb_push(self.__buffer, ctypes.POINTER(ctypes.c_uint8)(buffer), len(data))
        del buffer

    def __del__(self):
        _rb_del(self.__buffer)
