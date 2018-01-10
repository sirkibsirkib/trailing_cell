# Trailing Cell

A set of connected `TcWriter<M>`s have any number (initially 0) of reading
`TcWriter<T,M>` objects for any types `T` (typically the same type T)
This is particularly useful when:
    - The data has the set of writers W and readers R, where W != R
    - The speed of reads is more vital than the speed of writes.
      eg: Writes are very rare
    - Its OK if read states are slightly stale 
It also has with it the nice properties of:
    - Granular control of reader-synchronization events
    - joining and leaving of writers whenever (using TcWriter::clone)
    - joining and leaving of readers whenever (using TcWriter::add_reader)
    - both blocking and non-blocking write options
    - a reader can be unwrapped to return their T state.

The implementation allows readers to be initialized with not only different
local states, but even initial states of different types, (so long as they
all implement TakesMessage<M> for the same M as the writer(s)). It's not
clear to me how this option might be useful, but it costs nothing to
support, so why not.

TcWriter is implemented as a wrapper over a `bus::Bus`, with 'write'
messages being broadcast to all readers, and readers explicitly accepting
messages and applying them to their local T state.

See the tests for some commented examples