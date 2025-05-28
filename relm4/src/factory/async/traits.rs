// Prevent false positive in nightly
#![allow(unused_qualifications)]

//! Traits for for managing and updating factories.

use crate::Sender;
use crate::channel::AsyncFactorySender;
use crate::factory::{AsyncPosition, DynamicIndex, FactoryView};
use crate::loading_widgets::LoadingWidgets;

use std::fmt::Debug;

/// A component that's stored inside a factory.
/// Similar to [`Component`](crate::Component) but adjusted to fit the life cycle
/// of factories.
pub trait AsyncFactoryComponent:
    AsyncPosition<<Self::ParentWidget as FactoryView>::Position> + Sized + 'static
{
    /// Container widget to which all widgets of the factory will be added.
    type ParentWidget: FactoryView + 'static;

    /// Messages which are received from commands executing in the background.
    type CommandOutput: Debug + Send + 'static;

    /// The message type that the factory component accepts as inputs.
    type Input: Debug + 'static;

    /// The message type that the factory component provides as outputs.
    type Output: Debug + 'static;

    /// The parameter used to initialize the factory component.
    type Init;

    /// The top-level widget of the factory component.
    type Root: AsRef<<Self::ParentWidget as FactoryView>::Children> + Debug + Clone;

    /// The type that's used for storing widgets created for this factory component.
    type Widgets: 'static;

    /// Initializes the model.
    fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> impl std::future::Future<Output = Self>;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Allows you to initialize the root widget with a temporary value
    /// as a placeholder until the [`init_model()`] future completes.
    ///
    /// This method does nothing by default.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    #[must_use]
    fn init_loading_widgets(_root: Self::Root) -> Option<LoadingWidgets> {
        None
    }

    /// Initializes the widgets.
    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncFactorySender<Self>,
    ) -> impl std::future::Future<Output = ()> {
        async {}
    }

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncFactorySender<Self>,
    ) -> impl std::future::Future<Output = ()> {
        async {}
    }

    /// Handles updates from a command.
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: AsyncFactorySender<Self>,
    ) -> impl std::future::Future<Output = ()> {
        async {
            self.update_cmd(message, sender.clone()).await;
            self.update_view(widgets, sender);
        }
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: AsyncFactorySender<Self>) {}

    /// Updates the model and view. Optionally returns a command to run.
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncFactorySender<Self>,
    ) -> impl std::future::Future<Output = ()> {
        async {
            self.update(message, sender.clone()).await;
            self.update_view(widgets, sender);
        }
    }

    /// Last method called before a factory component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}

    /// An identifier for the component used for debug logging.
    ///
    /// The default implementation of this method uses the address of the component, but
    /// implementations are free to provide more meaningful identifiers.
    fn id(&self) -> String {
        format!("{:p}", &self)
    }
}
