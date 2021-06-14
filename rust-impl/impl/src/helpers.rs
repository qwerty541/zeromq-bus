use core::panic;
use std::convert::From;
use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Instant;
use zmq::Socket;

//-----------------------------------------------------------------------------------------
// DeadLockSafeRwLock
//-----------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DeadLockSafeRwLock<T>(Arc<RwLock<T>>);

impl<T> DeadLockSafeRwLock<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn read<F: FnOnce(&T) -> O + 'static, O: 'static>(&self, function: F) -> O {
        let read_guard = self.0.read().unwrap_or_else(|_| panic!("rw lock poisoned"));

        function(&*read_guard)
    }

    pub fn write<F: FnOnce(&mut T) -> O + 'static, O: 'static>(&self, function: F) -> O {
        let mut write_guard = self
            .0
            .write()
            .unwrap_or_else(|_| panic!("rw lock poisoned"));

        function(&mut *write_guard)
    }
}

impl<T: Default> Default for DeadLockSafeRwLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for DeadLockSafeRwLock<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

//-----------------------------------------------------------------------------------------
// DeadLockSafeMutex
//-----------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DeadLockSafeMutex<T>(Arc<Mutex<T>>);

impl<T> DeadLockSafeMutex<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn lock<F: FnOnce(&mut T) -> O + 'static, O: 'static>(&self, function: F) -> O {
        let mut lock_guard = self
            .0
            .lock()
            .unwrap_or_else(|_| panic!("mutex lock poisoned"));

        function(&mut *lock_guard)
    }
}

impl<T: Default> Default for DeadLockSafeMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for DeadLockSafeMutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

//-----------------------------------------------------------------------------------------
// BusPublisherData
//-----------------------------------------------------------------------------------------

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
