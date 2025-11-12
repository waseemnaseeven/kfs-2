use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

/// A minimal, non-recursive spinlock.
/// - Provides Acquire/Release semantics for mutual exclusion.
/// - Not fair (may starve under heavy contention).
/// - Not IRQ-safe: do **not** use from both normal and interrupt context on the same CPU without an IRQ-disabling wrapper.
/// - Intended for early boot or very short critical sections.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// Soundness:
// - Interior mutability is gated by the lock bit.
// - Requiring T: Send ensures moving the protected data across threads is sound.
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Blocks until the lock is acquired. Non-recursive.
    pub fn lock(&self) -> SpinGuard<'_, T> {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinGuard {
            lock: self,
            _nosend: PhantomData,
        }
    }

    /// Attempts to acquire the lock without blocking.
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinGuard {
                lock: self,
                _nosend: PhantomData,
            })
        } else {
            None
        }
    }

    /// Returns whether the lock bit is set (best-effort; do not rely for logic).
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    fn unlock(&self) {
        // Release publishes all writes to the protected data before clearing the bit.
        self.locked.store(false, Ordering::Release);
    }
}

/// RAII guard for `SpinLock`.
pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
    // Encodes an exclusive borrow of T for 'a → naturally !Send and !Sync.
    _nosend: PhantomData<&'a mut T>,
}

impl<'a, T> core::ops::Deref for SpinGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // Safety: we hold the lock; exclusive access is enforced by the protocol.
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SpinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: we hold the lock exclusively; no aliasing mutable refs.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
