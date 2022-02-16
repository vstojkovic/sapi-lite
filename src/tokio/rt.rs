#[cfg_attr(docsrs, doc(cfg(feature = "tokio-rt")))]
/// An extension for [`tokio::runtime::Builder`] to enable SAPI.
///
/// This trait is [sealed](https://rust-lang.github.io/api-guidelines/future-proofing.html).
pub trait BuilderExt: private::Sealed {
    /// Ensures that every thread spawned by the runtime will initialize SAPI when started and
    /// deinitialize it when stopped.
    fn enable_sapi(&mut self) -> &mut Self;
}

impl BuilderExt for tokio::runtime::Builder {
    fn enable_sapi(&mut self) -> &mut Self {
        self.on_thread_start(|| {
            let _ = crate::initialize();
        })
        .on_thread_stop(crate::finalize)
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for tokio::runtime::Builder {}
}
