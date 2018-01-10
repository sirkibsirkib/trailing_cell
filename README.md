# Trailing Cell

## Purpose
This little project is inspired by works such as `magnetic` and `evmap`, which concentrate on lock-freedom. Here I introduce some wrapper representing a sort of `Cell` that implmenets 'trailing state' semantics. This structure is useful in the situation you have a performance-critical data structure for which reads must be fast, but writes can be slow (perhaps they are very rare).

For instance, I intend to use this for a game to store a `bidir-map` between client ID's of which there is a fixed amount) and player ID's (which are tied to players, like usernames).

## Usefulness

A set of connected `TcWriter<M>`s have any number (initially 0) of reading
`TcWriter<T,M>` objects for any types `T` (typically the same type T)
This is particularly useful when:
 * The data has the set of writers W and readers R, where W != R
 * The speed of reads is more vital than the speed of writes.
      eg: Writes are very rare
 * Its OK if read states are slightly stale
 
It also has with it the nice properties of:
 * Granular control of reader-synchronization events
 * joining and leaving of writers whenever (using `TcWriter::clone`)
 * joining and leaving of readers whenever (using `TcWriter::add_reader`)
 * both blocking and non-blocking write options
 * a reader can be unwrapped to return their `T` state.

The implementation allows readers to be initialized with not only different
local states, but even initial states of different types, (so long as they
all implement `TakesMessage<M>` for the same `M` as the writer(s)). It's not
clear to me how this option might be useful, but it costs nothing to
support, so why not.

## Implementation

TcWriter is implemented as a wrapper over a `bus::Bus`, with 'write'
messages being broadcast to all readers, and readers explicitly accepting
messages and applying them to their local `T` state.


## Example

```
use std::time::Duration;
use std::thread;

let w = TcWriter::new(10);
let mut r = w.add_reader(vec![]);
let ten_millis = Duration::from_millis(10);
let mut handles = vec![];
// this just goes to show how to spread TcReaders over threads
for i in 0..5 {
	let w_clone = w.clone();
	handles.push(thread::spawn(move || {
		thread::sleep(ten_millis);
		w_clone.clone().apply_change(Change::Push(i));
		thread::sleep(ten_millis);
	}));
}
// r's state is still stale, regardless of what the writers are doing.
assert_eq!(r.get_mut_stale().len(), 0);
thread::sleep(ten_millis);
// We can't statically make any guarantees here except that the number 
// of Push messages that have arrives lies on the interval [0,5]
assert!(r.get_mut_fresh().len() <= 5);
for h in handles {
	h.join().is_ok();
}
r.update();
// Now we can be sure all 5 messages have arrived, but we don't know the
// order of vector elements.
assert_eq!(r.get_mut_stale().len(), 5);

```

See the tests for some more commented examples