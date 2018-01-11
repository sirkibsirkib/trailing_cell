# Trailing Cell

## Purpose
This little project is inspired by works such as `magnetic` and `evmap`, which concentrate on lock-freedom. Here I introduce some wrapper representing a sort of `Cell` that implmenets 'trailing state' semantics. This structure is useful in the situation you have a performance-critical data structure for which reads must be fast, but writes can be slow (perhaps they are very rare). The key here is that the state is provided by the user, and under the user's control.

For instance, I intend to use this for a game to store a `bidir-map` between client ID's of which there is a fixed amount) and player ID's (which are tied to players, like usernames).

## Usefulness

A set of connected `TcWriter<M>`s have any number (initially 0) of reading
`TcReader<T,M>` objects for any types `T` (typically the same type T).
This is particularly useful when:
 * The wrapped `T` data has the set of writers W and readers R, where W != R.
 * The speed of reads is more vital than the speed of writes.
      eg: writes are very rare.
 * It's OK if read states are slightly stale.
 
It also has with it the nice properties of:
 * Granular control of reader-synchronization events.
 * Joining and leaving of writers whenever (using `TcWriter::clone`).
 * Joining and leaving of readers whenever (using `TcWriter::add_reader`).
 * Both blocking and non-blocking write options.
 * A reader can be unwrapped to return their `T` state.

The implementation allows readers to be initialized with not only different
local states, but even initial states of different types, (so long as they
all implement `TakesMessage<M>` for the same `M` as the writer(s)). It's not
clear to me how this option might be useful, but it costs nothing to
support, so why not.

## Using It Yourself

For the writers and readers to communicate, they rely on some concept of 'message'. As such, before one can do anything, one needs to implement `trait TakesMessage` for the objects that are going to represent 'state'. This involves implementing only one function, which defines how your state object is updated when faced with a particular message.

All that remains then is to create a first writer object. All readers connected to it descend from this writer, either directly or from other writers that descend from it. These readers can then be thrown around on threads as desired, each calling whichever functions necessary, all ultimately boiling down to:
 * Synchronize with writer(s).
 * Access the inner data.

## Underlying Implementation

`TcWriter` and `TcReader` are implemented as wrappers over a `bus::Bus`, where writers act as `M` producers, and readers act as `M` consumers. However, a message from any writer arrives at all readers. Readers consume buffered messages when they call their `TcReader::update` function, and serially apply these messages to their local state (as defined by the trait).


## Example

```rust

```

See `tests.rs` for more annotated examples.