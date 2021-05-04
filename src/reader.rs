use crate::client::DeserializeMessage;
use crate::consumer::{AckMessage, ConsumerOptions, DeadLetterPolicy, Message, TopicConsumer};
use crate::error::{ConsumerError, Error};
use crate::executor::Executor;
use crate::message::proto::command_subscribe::SubType;
use crate::Pulsar;
use chrono::{DateTime, Utc};
use futures::task::{Context, Poll};
use futures::{Future, SinkExt, Stream, StreamExt};
use futures::channel::mpsc::SendError;
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
    consumer: TopicConsumer<T, Exe>,
    state: Option<State<T>>,
}

impl<T: DeserializeMessage + 'static, Exe: Executor> Unpin for Reader<T, Exe> {}

enum State<T: DeserializeMessage> {
    PollingConsumer,
    PollingAck(Message<T>, Pin<Box<dyn Future<Output = Result<(), SendError>>>>),
}

impl<T: DeserializeMessage + 'static, Exe: Executor> Stream for Reader<T, Exe> {
    type Item = Result<Message<T>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.state.take().unwrap() {
            State::PollingConsumer => {
                match Pin::new(&mut this.consumer).poll_next(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Ready(Some(Ok(msg))) => {
                        let mut acker = this.consumer.acker();
                        let message_id = msg.message_id.clone();
                        this.state = Some(State::PollingAck(msg, Box::pin(async move {
                            acker.send(AckMessage::Ack(message_id, false)).await
                        })));
                        return Pin::new(this).poll_next(cx);
                    }
                    Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                }
            }
            State::PollingAck(msg, mut ack_fut) => match ack_fut.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(res) => {
                    this.state = Some(State::PollingConsumer);
                    return Poll::Ready(Some(res.map_err(|err| Error::Consumer(err.into())).map(|()| msg)));
                }
            },
        }
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

    /// returns topic this reader is subscribed on
    pub fn topic(&self) -> String {
        self.consumer.topic()
    }

    /// returns a list of broker URLs this reader is connnected to
    pub fn connections(&self) -> &Url {
        self.consumer.connection.url()
    }
    /// returns the consumer's configuration options
    pub fn options(&self) -> &ConsumerOptions {
        &self.consumer.config.options
    }

    // is this necessary?
    /// returns the consumer's dead letter policy options
    pub fn dead_letter_policy(&self) -> Option<&DeadLetterPolicy> {
        self.consumer.dead_letter_policy.as_ref()
    }

    /// returns the readers's subscription name
    pub fn subscription(&self) -> &str {
        &self.consumer.config.subscription
    }
    /// returns the reader's subscription type
    pub fn sub_type(&self) -> SubType {
        self.consumer.config.sub_type
    }

    /// returns the reader's batch size
    pub fn batch_size(&self) -> Option<u32> {
        self.consumer.config.batch_size
    }

    /// returns the reader's name
    pub fn reader_name(&self) -> Option<&str> {
        self.consumer
            .config
            .consumer_name
            .as_ref()
            .map(|s| s.as_str())
    }

    /// returns the reader's id
    pub fn reader_id(&self) -> u64 {
        self.consumer.consumer_id
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
