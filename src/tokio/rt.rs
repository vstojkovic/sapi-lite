pub trait BuilderExt: private::Sealed {
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
