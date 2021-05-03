use futures::task::{Context, Poll};
use futures::{
    channel::{mpsc, oneshot},
    Future, FutureExt, SinkExt, Stream, StreamExt,
};
use std::pin::Pin;

use crate::client::DeserializeMessage;
use crate::consumer::{Consumer, Message};
use crate::error::{ConnectionError, ConsumerError, Error};
use crate::executor::Executor;
use crate::Pulsar;
use regex::Regex;

/// Configuration options for readers
#[derive(Default, Debug)]
pub struct ReaderOptions {
    // to be filled
}

pub struct Reader<T: DeserializeMessage, Exe: Executor> {
    consumer: Consumer<T, Exe>,
}

impl<T: DeserializeMessage + 'static, Exe: Executor> Stream for Reader<T, Exe> {
    type Item = Result<Message<T>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.consumer).poll_next(cx)
    }
}

impl<T: DeserializeMessage, Exe: Executor> Reader<T, Exe> {
    pub fn builder(pulsar: &Pulsar<Exe>) -> ReaderBuilder<Exe> {
        ReaderBuilder::new(pulsar)
    }
}

/// Builder structures for readers
///
/// This is the way to create a Reader
#[derive(Clone)]
pub struct ReaderBuilder<Exe: Executor> {
    pulsar: Pulsar<Exe>,
    topics: Option<Vec<String>>,
    topic_regex: Option<Regex>,
}

impl<Exe: Executor> ReaderBuilder<Exe> {
    /// creates a new [ReaderBuilder] from an existing client instance
    pub fn new(pulsar: &Pulsar<Exe>) -> Self {
        ReaderBuilder {
            pulsar: pulsar.clone(),
            topics: None,
            topic_regex: None,
        }
    }
}
