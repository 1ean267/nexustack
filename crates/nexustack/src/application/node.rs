/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart,
    application::{
        ApplicationPartBuilder, Chain,
        chain::{InHead, InTail, Index},
        configurable::Configurable,
        instrumentation::WithInstrumentation,
    },
    inject::{ConstructionResult, ServiceProvider},
};
use either::Either;
use futures_util::TryFutureExt;
use std::{any::TypeId, borrow::Cow};
use tokio_util::sync::CancellationToken;

/// A composite application part that combines two other application parts, `Head` and `Tail`.
///
/// The `Node` struct allows you to chain two application parts together, enabling them to
/// work as a single unit. Each part must implement the `ApplicationPart` trait, and the
/// `Node` struct itself implements `ApplicationPart`, delegating its behavior to the
/// `Head` and `Tail` components.
///
/// # Type Parameters
/// - `Head`: The first application part in the chain.
/// - `Tail`: The second application part in the chain.
///
pub struct Node<Head, Tail> {
    pub(crate) head: Head,
    pub(crate) tail: Tail,
}

impl<Head, Tail> ApplicationPart for Node<Head, Tail>
where
    Head: ApplicationPart + Send + Sync,
    Tail: ApplicationPart + Send + Sync,
{
    type Error = Either<Head::Error, Tail::Error>;

    fn name() -> Cow<'static, str> {
        match (Head::name(), Tail::name()) {
            (Cow::Borrowed(head), Cow::Borrowed(tail)) => Cow::Owned(format!("{head}, {tail}")),
            (Cow::Borrowed(head), Cow::Owned(mut tail)) => {
                if (tail.capacity() - tail.len()) >= (head.len() + 2) {
                    tail.insert_str(0, ", ");
                    tail.insert_str(0, head);
                    Cow::Owned(tail)
                } else {
                    // We need to reallocate anyway, so just use format!
                    Cow::Owned(format!("{head}, {tail}"))
                }
            }
            (Cow::Owned(mut head), Cow::Borrowed(tail)) => {
                head.push(',');
                head.push(' ');
                head.push_str(tail);
                Cow::Owned(head)
            }
            (Cow::Owned(mut head), Cow::Owned(tail)) => {
                head.push(',');
                head.push(' ');
                head.push_str(tail.as_str());
                Cow::Owned(head)
            }
        }
    }

    async fn before_startup(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        tokio::try_join!(
            self.head
                .before_startup(cancellation_token.clone())
                .map_err(Either::Left),
            self.tail
                .before_startup(cancellation_token)
                .map_err(Either::Right)
        )
        .map(|_| ())
    }

    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        tokio::try_join!(
            self.head
                .run(cancellation_token.clone())
                .map_err(Either::Left),
            self.tail.run(cancellation_token).map_err(Either::Right)
        )
        .map(|_| ())
    }

    async fn before_shutdown(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        tokio::try_join!(
            self.head
                .before_shutdown(cancellation_token.clone())
                .map_err(Either::Left),
            self.tail
                .before_shutdown(cancellation_token)
                .map_err(Either::Right)
        )
        .map(|_| ())
    }
}

impl<Head, Tail> ApplicationPartBuilder for Node<Head, Tail>
where
    Head: ApplicationPartBuilder,
    Tail: ApplicationPartBuilder,
{
    type ApplicationPart = Node<WithInstrumentation<Head::ApplicationPart>, Tail::ApplicationPart>;

    fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart> {
        Ok(Node {
            head: WithInstrumentation(self.head.build(service_provider.clone())?),
            tail: self.tail.build(service_provider)?,
        })
    }
}

impl<Head, Tail> Configurable<'static> for Node<Head, Tail>
where
    Head: ApplicationPartBuilder + 'static,
    Tail: ApplicationPartBuilder + Configurable<'static>,
{
    fn has_item<I: 'static>() -> bool {
        // TODO: This is a nasty hack. Can we find another way?
        TypeId::of::<Head>() == TypeId::of::<I>() || <Tail as Configurable<'_>>::has_item::<I>()
    }
}

impl<Head, Tail, HeadIndex> Chain<InHead<HeadIndex>> for Node<Head, Tail>
where
    HeadIndex: Index,
    Head: Chain<HeadIndex>,
{
    type Element = Head::Element;
    fn get(&self) -> &Self::Element {
        self.head.get()
    }

    fn get_mut(&mut self) -> &mut Self::Element {
        self.head.get_mut()
    }
}

impl<Head, Tail, TailIndex> Chain<InTail<TailIndex>> for Node<Head, Tail>
where
    TailIndex: Index,
    Tail: Chain<TailIndex>,
{
    type Element = Tail::Element;
    fn get(&self) -> &Self::Element {
        self.tail.get()
    }

    fn get_mut(&mut self) -> &mut Self::Element {
        self.tail.get_mut()
    }
}
