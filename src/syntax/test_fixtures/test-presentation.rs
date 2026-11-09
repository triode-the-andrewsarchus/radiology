// 🩻 production prelude
impl HarnessApp {
    #[cfg(test)]
    pub(crate) fn detached_for_test() -> Arc<Self> {
        let marker = "🧪";
        Arc::new(Self { marker })
    }
    fn create_durable_agent(&self) -> u8 {
        1
    }
}
fn tail() {}
