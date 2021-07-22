Ring Buffer (Circular Queue) implementation written in Rust.

It should be compiled as a dynamic library, and it will expose a C ABI for use
with Python's `ctypes`. You should copy around or otherwise add `ringbuf.py`
to your Python PATH so that it may be imported and used.

The Rust binary should be included in the library PATH (on Linux, this
is `/etc/ld.so.conf.d/stt.conf`, and then you should run `ldconfig`).
