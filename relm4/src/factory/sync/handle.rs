use std::fmt;

use crate::Sender;
use crate::factory::{DataGuard, FactoryComponent, FactoryView};

/// Don't allow public access to a [`FactoryHandle`].
///
/// It might be unsafe to extract `data` or `runtime`.
/// Inside this type, it is guaranteed that extracting `data` will drop `runtime` before to
/// comply with all required safety guarantees.
pub(super) struct FactoryHandle<C: FactoryComponent> {
    pub(super) data: DataGuard<C, C::Widgets, C::Output>,
    pub(super) root_widget: C::Root,
    pub(super) returned_widget: <C::ParentWidget as FactoryView>::ReturnedWidget,
    pub(super) input: Sender<C::Input>,
    pub(super) notifier: Sender<()>,
}

impl<C: FactoryComponent> fmt::Debug for FactoryHandle<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FactoryHandle")
            .field("data", &"<FactoryComponent>")
            .field("root_widget", &self.root_widget)
            .field("input", &self.input)
            .field("notifier", &self.notifier)
            .finish()
    }
}
