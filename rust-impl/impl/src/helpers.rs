use core::panic;
use std::convert::Into;
use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use zmq::Socket;

#[derive(Debug, Clone)]
pub struct MutexLock<T> {
    lock: Arc<Mutex<T>>,
    identifier: String,
}

impl<T: 'static> MutexLock<T> {
    pub fn new<I: Into<String>>(value: T, identifier: I) -> Self {
        Self {
            lock: Arc::new(Mutex::new(value)),
            identifier: identifier.into(),
        }
    }

    pub fn lock<F: FnOnce(&mut T) -> O + 'static, O: 'static>(&self, function: F) -> O {
        let mut lock_guard = self
            .lock
            .lock()
            .unwrap_or_else(|error| panic!("failed to lock mutex because of: {}", error));
        function(&mut *lock_guard)
    }
}

#[must_use]
pub struct BusPublisherData {
    socket: Socket,
    last_action_time: Instant,
}

impl BusPublisherData {
    pub fn new(socket: Socket) -> Self {
        Self {
            socket,
            last_action_time: Instant::now(),
        }
    }

    #[must_use]
    pub fn get_last_action_time(&self) -> Instant {
        self.last_action_time
    }

    pub fn update_last_action_time(&mut self) {
        self.last_action_time = Instant::now();
    }
}

impl Deref for BusPublisherData {
    type Target = Socket;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl DerefMut for BusPublisherData {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.socket
    }
}

impl fmt::Debug for BusPublisherData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BusPublisherData")
            .field("socket", &self.socket.get_socket_type())
            .field("last_action_time", &self.last_action_time)
            .finish()
    }
}
