use windows::core::{Interface, Result};
use windows_future::AsyncStatus;

mod waiter {
    use windows::Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::Threading::{CreateEventW, SetEvent, WaitForSingleObject},
    };

    pub struct Waiter(HANDLE);
    pub struct WaiterSignaler(HANDLE);
    unsafe impl Send for WaiterSignaler {}

    impl Waiter {
        pub fn new() -> crate::Result<(Self, WaiterSignaler)> {
            unsafe {
                let handle = CreateEventW(core::ptr::null(), 1, 0, core::ptr::null());
                if handle.is_null() {
                    Err(crate::Error::from_thread())
                } else {
                    Ok((Self(handle), WaiterSignaler(handle)))
                }
            }
        }
    }

    impl WaiterSignaler {
        /// # Safety
        /// Signals the `Waiter`. This is unsafe because the lifetime of `WaiterSignaler` is not tied
        /// to the lifetime of the `Waiter`. This is not possible in this case because the `Waiter`
        /// is used to signal a WinRT async completion and the compiler doesn't know that the lifetime
        /// of the delegate is bounded by the calling function.
        pub unsafe fn signal(&self) {
            // https://github.com/microsoft/windows-rs/pull/374#discussion_r535313344
            unsafe {
                SetEvent(self.0);
            }
        }
    }

    impl Drop for Waiter {
        fn drop(&mut self) {
            unsafe {
                WaitForSingleObject(self.0, 0xFFFFFFFF);
                CloseHandle(self.0);
            }
        }
    }
}

pub trait Async: Interface {
    // The type of value produced on completion.
    type Output: Clone;

    // The type of the delegate use for completion notification.
    type CompletedHandler: Interface;

    // Sets the handler or callback to invoke when execution completes. This handler can only be set once.
    fn set_completed<F: Fn(&Self) + Send + 'static>(&self, handler: F) -> Result<()>;

    // Calls the given handler with the current object and status.
    #[cfg(feature = "std")]
    fn invoke_completed(&self, handler: &Self::CompletedHandler, status: AsyncStatus);

    // Returns the value produced on completion. This should only be called when execution completes.
    fn get_results(&self) -> Result<Self::Output>;

    // Gets the current status of async execution. This calls `QueryInterface` so should be used sparingly.
    fn status(&self) -> Result<AsyncStatus>;

    // Waits for the async execution to finish and then returns the results.
    fn join(&self) -> Result<Self::Output> {
        if self.status()? == AsyncStatus::Started {
            let (_waiter, signaler) = Waiter::new()?;
            self.set_completed(move |_| {
                // This is safe because the waiter will only be dropped after being signaled.
                unsafe {
                    signaler.signal();
                }
            })?;
        }
        self.get_results()
    }

    // Calls `op(result)` when async execution completes.
    fn when<F>(&self, op: F) -> Result<()>
    where
        F: FnOnce(Result<Self::Output>) + Send + 'static,
    {
        if self.status()? == AsyncStatus::Started {
            // The `set_completed` closure is guaranteed to only be called once, like `FnOnce`, by the async pattern,
            // but Rust doesn't know that so `RefCell` is used to pass `op` in to the closure.
            let op = core::cell::RefCell::new(Some(op));
            self.set_completed(move |sender| {
                if let Some(op) = op.take() {
                    op(sender.get_results());
                }
            })?;
        } else {
            op(self.get_results());
        }
        Ok(())
    }
}
