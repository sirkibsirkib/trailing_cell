use std::cell::Cell;
use std::sync::{Arc,Mutex};

extern crate bus;
use bus::{Bus,BusReader};

#[cfg(test)]
mod tests;

/// Prerequisite trait for any struct to be the 'wrapped' value of a TcReader
pub trait TakesMessage<M> {
    fn take_message(&mut self, t: M);
}

/// A set of connected `TcWriter<M>`s have any number (initially 0) of reading
/// TcWriter<T,M> objects for any types `T` (typically the same type T)
///
/// This is particularly useful when:
///     - The data has the set of writers W and readers R, where W != R
///     - The speed of reads is more vital than the speed of writes.
///       eg: Writes are very rare
///     - Its OK if read states are slightly stale 
///
/// It also has with it the nice properties of:
///     - Granular control of reader-synchronization events
///     - joining and leaving of writers whenever (using TcWriter::clone)
///     - joining and leaving of readers whenever (using TcWriter::add_reader)
///     - both blocking and non-blocking write options
///     - a reader can be unwrapped to return their T state.
/// 
/// The implementation allows readers to be initialized with not only different
/// local states, but even initial states of different types, (so long as they
/// all implement TakesMessage<M> for the same M as the writer(s)). It's not
/// clear to me how this option might be useful, but it costs nothing to
/// support, so why not.
///
/// TcWriter is implemented as a wrapper over a `bus::Bus`, with 'write'
/// messages being broadcast to all readers, and readers explicitly accepting
/// messages and applying them to their local T state.
///
/// See the tests for some commented examples
pub struct TcWriter<M>
where M: Sync + Clone {
    producer: Arc<Mutex<Bus<M>>>,
    // phantom: PhantomData<T>,
}

impl<M> TcWriter<M>
where M: Sync + Clone {

    /// Constructs a new `TcWriter<T,D>`.
    /// Facilitates mutation of the wrapped T object.
    #[inline]
    pub fn new(capacity: usize) -> Self {
        TcWriter { producer: Arc::new(Mutex::new(Bus::new(capacity))), }
    }

    /// Broadcasts the given D message to readers. Blocks until there is space
    /// on bus. has the same semantics as underlying `bus::Bus::broadcast`
    pub fn apply_change(&self, m: M) {
        self.producer.lock().unwrap().broadcast(m)
    }

    /// Broadcasts the given D message to readers, returns immediately if bus is
    /// full. Has the same semantics as underlying `bus::Bus::try_broadcast`
    pub fn try_apply_change(&self, m: M) -> Result<(), M> {
        if let Err(m) = self.producer.lock().unwrap().try_broadcast(m) {
            Err(m)
        } else {
            Ok(())
        }
    }

    /// Creates, registers and returns a new reader object to the underlying `T`
    /// The reader will maintain its own state
    pub fn add_reader<T: TakesMessage<M>>(&self, init: T) -> TcReader<T, M> {
        TcReader {
            data: Cell::new(init),
            consumer: self.producer.lock().unwrap().add_rx()
        }
    }
}

impl<M> Clone for TcWriter<M>
where M: Sync + Clone {
    fn clone(&self) -> Self {
        TcWriter { producer: self.producer.clone(), }
    }

    fn clone_from(&mut self, source: &Self) {
        self.producer = source.producer.clone();
    }
}


/// `TcReader<T,M>` maintains its local `T` object. The reader will receive and 
/// apply incoming `M` messages to its T whenever explicitly invoked by
/// `update`, `update_limited` or implicity by `get_mut_fresh`.
/// 
/// Access to the local copy is granted through the two `get_mut` variants.
/// Without any messages, this local access is very fast. The reader can also be
/// consumed to unwrap the local `T`.
/// 
/// The very explicit convention of using `stale` and `fresh` everywhere is to
/// make unintentionally forgetting a crucial update less likely.
pub struct TcReader<T,M>
where T: TakesMessage<M> {
    data: Cell<T>,
    consumer: BusReader<M>,
}

impl<T,M> TcReader<T,M>
where T: TakesMessage<M>,
      M: Sync + Clone {
    
    /// Receives all waiting messages and applies them to the local object.
    pub fn update(&mut self) {
        while let Ok(msg) = self.consumer.try_recv() {
            unsafe { (*self.data.as_ptr()).take_message(msg); }
        }
    }

    /// Receives any waiting messages up to a limit.
    /// Returns number of messages received and appllied
    pub fn update_limited(&mut self, limit: usize) -> usize {
        let mut count = 0;
        for _ in 0..limit {
            if let Ok(msg) = self.consumer.try_recv() {
                unsafe { (*self.data.as_ptr()).take_message(msg); }
                count += 1;
            } else { break }
        }
        count
    }

    /// Returns a mutable reference to the inner object, after applying updates.
    /// Use this when freshness is more vital than speed.
    pub fn get_mut_fresh(&mut self) -> &mut T {
        self.update();
        self.data.get_mut()
    }
    
    /// Returns a mutable reference to the inner object, without applying
    /// any updates. Use this when speed is more vital than freshness.
    pub fn get_mut_stale(&mut self) -> &mut T {
        self.data.get_mut()
    }

    /// Consumes the reader, returning the current version of the trailing `T`.
    pub fn into_inner_stale(self) -> T {
        self.data.into_inner()
    }

    /// Consumes the reader, returning the trailing `T`, but applying all
    /// waiting messages first
    pub fn into_inner_fresh(mut self) -> T {
        self.update();
        self.data.into_inner()
    }
}

