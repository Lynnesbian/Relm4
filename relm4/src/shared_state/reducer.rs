use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::{RUNTIME, Sender};

use super::SubscriberFn;

/// A trait that implements a reducer function.
///
/// For more information, see [`Reducer`].
pub trait Reducible {
    /// The input message type used to modify the data.
    type Input;

    /// Initialize the data.
    fn init() -> Self;

    /// Process the input message and update the state.
    ///
    /// Return [`true`] to notify all subscribers.
    /// Return [`false`] to ignore all subscribers.
    ///
    /// For example, it makes sense to return [`false`] to indicate
    /// that the message had no (noteworthy) effect on the data and
    /// the subscribers don't need to be notified.
    fn reduce(&mut self, input: Self::Input) -> bool;
}

struct ReducerInner<Data: Reducible> {
    sender: Sender<Data::Input>,
    subscribers: Arc<RwLock<Vec<SubscriberFn<Data>>>>,
}

impl<Data> Default for ReducerInner<Data>
where
    Data: Reducible + Send + 'static,
    Data::Input: Send,
{
    fn default() -> Self {
        let (sender, receiver) = crate::channel();
        let subscribers: Arc<RwLock<Vec<SubscriberFn<Data>>>> = Arc::default();

        let rt_subscribers = subscribers.clone();
        RUNTIME.spawn(async move {
            let mut data = Data::init();
            while let Some(input) = receiver.recv().await {
                if data.reduce(input) {
                    // Remove all elements which had their senders dropped.
                    rt_subscribers
                        .write()
                        .unwrap()
                        .retain(|subscriber| subscriber(&data));
                }
            }
        });

        Self {
            sender,
            subscribers,
        }
    }
}

impl<Data> std::fmt::Debug for ReducerInner<Data>
where
    Data: std::fmt::Debug + Reducible,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReducerInner")
            .field("sender", &self.sender)
            .field("subscribers", &self.subscribers.try_read().map(|s| s.len()))
            .finish()
    }
}

/// A type that allows you to share information across your
/// application easily.
///
/// Reducers receive messages, update their state accordingly
/// and notify their subscribers.
///
/// Unlike [`SharedState`](super::SharedState), this type doesn't
/// allow direct access to the internal data.
/// Instead, it updates its state after receiving messages, similar to components.
/// After the message is processed, all subscribers will be notified.
///
/// # Example
///
/// ```
/// use relm4::{Reducer, Reducible};
///
/// struct CounterReducer(u8);
///
/// enum CounterInput {
///     Increment,
///     Decrement,
/// }
///
/// impl Reducible for CounterReducer {
///     type Input = CounterInput;
///
///     fn init() -> Self {
///         Self(0)
///     }
///
///     fn reduce(&mut self, input: Self::Input) -> bool {
///         match input {
///             CounterInput::Increment => {
///                 self.0 += 1;
///             }
///             CounterInput::Decrement =>  {
///                 self.0 -= 1;
///             }
///         }
///         true
///     }
/// }
///
/// // Create the reducer.
/// static REDUCER: Reducer<CounterReducer> = Reducer::new();
///
/// // Update the reducer.
/// REDUCER.emit(CounterInput::Increment);
/// # use std::time::Duration;
/// # std::thread::sleep(Duration::from_millis(10));
///
/// // Create a channel and subscribe to changes.
/// let (sender, receiver) = relm4::channel();
/// REDUCER.subscribe(&sender, |data| data.0);
///
/// // Count up to 2.
/// REDUCER.emit(CounterInput::Increment);
/// assert_eq!(receiver.recv_sync().unwrap(), 2);
/// ```
#[derive(Debug)]
pub struct Reducer<Data: Reducible> {
    inner: Lazy<ReducerInner<Data>>,
}

impl<Data> Default for Reducer<Data>
where
    Data: Reducible + Send + 'static,
    Data::Input: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Data> Reducer<Data>
where
    Data: Reducible + Send + 'static,
    Data::Input: Send,
{
    /// Create a new [`Reducer`] variable.
    ///
    /// The data will be initialized lazily on the first access.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: Lazy::new(ReducerInner::default),
        }
    }

    /// Subscribe to a [`Reducer`].
    /// Any subscriber will be notified with a message every time
    /// you modify the reducer (by calling [`Self::emit()`]).
    pub fn subscribe<Msg, F>(&self, sender: &Sender<Msg>, f: F)
    where
        F: Fn(&Data) -> Msg + 'static + Send + Sync,
        Msg: Send + 'static,
    {
        let sender = sender.clone();
        self.inner
            .subscribers
            .write()
            .unwrap()
            .push(Box::new(move |data: &Data| {
                let msg = f(data);
                sender.send(msg).is_ok()
            }));
    }

    /// An alternative version of [`subscribe()`](Self::subscribe()) that only send a message if
    /// the closure returns [`Some`].
    pub fn subscribe_optional<Msg, F>(&self, sender: &Sender<Msg>, f: F)
    where
        F: Fn(&Data) -> Option<Msg> + 'static + Send + Sync,
        Msg: Send + 'static,
    {
        let sender = sender.clone();
        self.inner
            .subscribers
            .write()
            .unwrap()
            .push(Box::new(move |data: &Data| {
                if let Some(msg) = f(data) {
                    sender.send(msg).is_ok()
                } else {
                    true
                }
            }));
    }

    /// Sends a message to the reducer to update its state.
    ///
    /// If the [`Reducible::reduce()`] method returns [`true`],
    /// all subscribers will be notified.
    pub fn emit(&self, input: Data::Input) {
        assert!(
            self.inner.sender.send(input).is_ok(),
            "Reducer runtime was dropped. Maybe a subscriber or the update function panicked?"
        );
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::{Reducer, Reducible};

    struct CounterReducer(u8);

    enum CounterInput {
        Increment,
        Decrement,
    }

    impl Reducible for CounterReducer {
        type Input = CounterInput;

        fn init() -> Self {
            Self(0)
        }

        fn reduce(&mut self, input: Self::Input) -> bool {
            match input {
                CounterInput::Increment => {
                    self.0 += 1;
                }
                CounterInput::Decrement => {
                    self.0 -= 1;
                }
            }
            true
        }
    }

    static REDUCER: Reducer<CounterReducer> = Reducer::new();

    #[test]
    fn shared_state() {
        // Count up to 3 and wait for events to be processed.
        REDUCER.emit(CounterInput::Increment);
        REDUCER.emit(CounterInput::Increment);
        REDUCER.emit(CounterInput::Increment);
        std::thread::sleep(Duration::from_millis(10));

        let (sender, receiver) = crate::channel();

        REDUCER.subscribe(&sender, |data| data.0);

        // Count up to 4 with receiver.
        REDUCER.emit(CounterInput::Increment);
        assert_eq!(receiver.recv_sync().unwrap(), 4);

        // Count down to 3.
        REDUCER.emit(CounterInput::Decrement);

        assert_eq!(receiver.recv_sync().unwrap(), 3);
    }
}
