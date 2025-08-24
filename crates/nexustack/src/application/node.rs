/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart,
    application::{ApplicationPartBuilder, configurable::Configurable},
    inject::{ConstructionResult, ServiceProvider},
};
use either::Either;
use futures_util::TryFutureExt;
use std::{any::Any, borrow::Cow};
use tokio_util::sync::CancellationToken;

pub(crate) struct Node<Head, Tail> {
    pub head: Head,
    pub tail: Tail,
}

impl<Head, Tail> ApplicationPart for Node<Head, Tail>
where
    Head: ApplicationPart + Sync,
    <Head as ApplicationPart>::Error: Send,
    Tail: ApplicationPart + Sync,
    <Tail as ApplicationPart>::Error: Send,
{
    type Error = Either<Head::Error, Tail::Error>;

    fn name(&self) -> Cow<'static, str> {
        match (self.head.name(), self.tail.name()) {
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
        &self,
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

    async fn run(&self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        tokio::try_join!(
            self.head
                .run(cancellation_token.clone())
                .map_err(Either::Left),
            self.tail.run(cancellation_token).map_err(Either::Right)
        )
        .map(|_| ())
    }

    async fn before_shutdown(
        &self,
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
    <Head as ApplicationPartBuilder>::ApplicationPart: Send + Sync,
    <<Head as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error: Send,
    Tail: ApplicationPartBuilder,
    <Tail as ApplicationPartBuilder>::ApplicationPart: Sync,
    <<Tail as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error: Send,
{
    type ApplicationPart = Node<Head::ApplicationPart, Tail::ApplicationPart>;

    fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart> {
        Ok(Node {
            head: self.head.build(service_provider.clone())?,
            tail: self.tail.build(service_provider)?,
        })
    }
}

impl<Head, Tail> Configurable<'static> for Node<Head, Tail>
where
    Head: ApplicationPartBuilder + 'static,
    Tail: ApplicationPartBuilder + Configurable<'static>,
{
    fn configure<I: 'static, C>(&mut self, configure: C) -> Result<(), C>
    where
        C: FnOnce(&mut I),
    {
        // This is not sound, can we make it sound?
        // if TypeId::of::<Head>() == TypeId::of::<I>() {
        //     let head = &mut self.head;
        //     let head = unsafe { &mut *(head as *mut Head as *mut I) };
        //     configure(head);
        //     Ok(())
        // }

        if let Some(head) = (&mut self.head as &mut dyn Any).downcast_mut::<I>() {
            configure(head);
            Ok(())
        } else {
            self.tail.configure::<I, C>(configure)
        }
    }
}
