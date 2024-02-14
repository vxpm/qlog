use std::fmt::Write;

/// Trait for types which can be logged.
pub trait Loggable: Send {
    /// Logs this value to a given writer.
    fn log_to(self, writer: &mut dyn Write) -> std::fmt::Result;
}

impl<F> Loggable for F
where
    F: Send + FnOnce(&mut dyn Write) -> std::fmt::Result,
{
    #[inline(always)]
    fn log_to(self, writer: &mut dyn Write) -> std::fmt::Result {
        self(writer)
    }
}

impl<'str> Loggable for &'str str {
    #[inline(always)]
    fn log_to(self, writer: &mut dyn Write) -> std::fmt::Result {
        writer.write_str(self)
    }
}

mod erased {
    use super::Loggable;
    use std::{alloc::Layout, fmt::Write, mem::MaybeUninit};

    const ERASED_SIZE: usize = 32;
    const INLINE_DATA_SIZE: usize = ERASED_SIZE - std::mem::size_of::<usize>();

    enum Inner {
        Inline {
            do_log_to: fn(*const (), &mut dyn Write) -> std::fmt::Result,
            data: MaybeUninit<[u8; INLINE_DATA_SIZE]>,
        },
        Boxed {
            do_log_to: fn(*const (), &mut dyn Write) -> std::fmt::Result,
            layout: Layout,
            data: *mut (),
        },
    }

    unsafe impl Send for Inner {}

    impl Inner {
        #[inline(always)]
        pub fn new<L>(value: L) -> Self
        where
            L: Loggable + 'static,
        {
            fn do_log_to<L>(value_ptr: *const L, writer: &mut dyn Write) -> std::fmt::Result
            where
                L: Loggable + 'static,
            {
                let value = unsafe { value_ptr.read_unaligned() };
                value.log_to(writer)
            }

            let do_log_to = unsafe { std::mem::transmute(do_log_to::<L> as fn(_, _) -> _) };
            if std::mem::size_of::<L>() > INLINE_DATA_SIZE {
                let layout = Layout::new::<L>();
                let data = unsafe { std::alloc::alloc(layout) };
                assert!(!data.is_null());

                unsafe { std::ptr::write_unaligned(data.cast(), value) };

                Self::Boxed {
                    do_log_to,
                    layout,
                    data: data.cast(),
                }
            } else {
                let mut data: MaybeUninit<[u8; ERASED_SIZE - 8]> = MaybeUninit::uninit();
                unsafe { std::ptr::write_unaligned(data.as_mut_ptr().cast(), value) };

                Self::Inline { do_log_to, data }
            }
        }
    }

    pub struct ErasedLoggable(Inner);

    impl ErasedLoggable {
        #[inline(always)]
        pub fn new<L>(value: L) -> Self
        where
            L: Loggable + 'static,
        {
            Self(Inner::new(value))
        }
    }

    impl Loggable for ErasedLoggable {
        #[inline(always)]
        fn log_to(self, writer: &mut dyn Write) -> std::fmt::Result {
            match self.0 {
                Inner::Inline { do_log_to, data } => {
                    do_log_to(std::ptr::addr_of!(data).cast(), writer)
                }
                Inner::Boxed {
                    do_log_to, data, ..
                } => do_log_to(data, writer),
            }
        }
    }

    impl Drop for ErasedLoggable {
        #[inline(always)]
        fn drop(&mut self) {
            if let Inner::Boxed { layout, data, .. } = self.0 {
                unsafe { std::alloc::dealloc(data.cast(), layout) };
            }
        }
    }
}

pub(crate) use erased::ErasedLoggable;
