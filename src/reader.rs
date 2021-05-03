use crate::client::DeserializeMessage;
use crate::consumer::{Consumer, ConsumerOptions, Message, DeadLetterPolicy};
use crate::error::{ConnectionError, ConsumerError, Error};
use crate::executor::Executor;
use crate::Pulsar;
use futures::task::{Context, Poll};
use futures::{
    channel::{mpsc, oneshot},
    Future, FutureExt, SinkExt, Stream, StreamExt,
};
use regex::Regex;
use std::pin::Pin;
use std::time::{Duration, Instant};
use url::Url;

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
    /// creates a [ReaderBuilder] from a client instance
    pub fn builder(pulsar: &Pulsar<Exe>) -> ReaderBuilder<Exe> {
        ReaderBuilder::new(pulsar)
    }

    /// test that the connections to the Pulsar brokers are still valid
    pub async fn check_connection(&self) -> Result<(), Error> {
        self.consumer.check_connection().await
    }

    // no ack methods - it's a reader!

    /// returns the list of topics this reader is subscribed on
    pub fn topics(&self) -> Vec<String> {
        self.consumer.topics()
    }

    /// returns a list of broker URLs this reader is connnected to
    pub fn connections(&self) -> Vec<&Url> {
        self.consumer.connections()
    }
    /// returns the consumer's configuration options
    pub fn options(&self) -> &ConsumerOptions {
        self.consumer.options()
    }

    // is this necessary?
    /// returns the consumer's dead letter policy options
    pub fn dead_letter_policy(&self) -> Option<&DeadLetterPolicy> {
        self.consumer.dead_letter_policy()
    }

    /// returns the readers's subscription name
    pub fn subscription(&self) -> &str {
        self.consumer.subscription()
    }
    /// returns the reader's subscription type
    pub fn sub_type(&self) -> SubType {
        self.consumer.sub_type()
    }

    /// returns the reader's batch size
    pub fn batch_size(&self) -> Option<u32> {
        self.consumer.batch_size()
    }

    /// returns the reader's name
    pub fn reader_name(&self) -> Option<&str> {
        self.consumer.consumer_name()
    }

    /// returns the reader's list of ids
    pub fn reader_id(&self) -> Vec<u64> {
        self.consumer.consumer_id()
    }

    /// returns the date of the last message reception
    pub fn last_message_received(&self) -> Option<DateTime<Utc>> {
        self.consumer.last_message_received()
    }

    /// returns the current number of messages received
    pub fn messages_received(&self) -> u64 {
        self.consumer.messages_received()
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
