## Pulsar
#### Future-based Rust bindings for [Apache Pulsar](https://pulsar.apache.org/)

[Documentation](https://docs.rs/pulsar)

Current status: Simple functionality, but expect API to change. Major API changes will come simultaneous with async-await stability, so look for that.
### Getting Started
Cargo.toml
```toml
futures = "0.1.23"
pulsar = "0.1.1"
tokio = "0.1.11"

```
main.rs
```rust
extern crate pulsar;

// if you want connection pooling
extern crate r2d2_pulsar;
extern crate r2d2;

// if you want serde
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
```
#### Producing
```rust
use tokio::runtime::Runtime;
use futures::future::{self, Future};
use pulsar::Producer;

pub struct SomeData {
    ...
}

fn serialize(data: &SomeData) -> Vec<u8> {
    ...
}

fn main() {
    let pulsar_addr = "...";
    let producer_name = "some_producer_name";
    let runtime = Runtime::new().unwrap();

    let producer = Producer::new(pulsar_addr, Some(producer_name.to_string()), None, None, runtime.executor())
       .wait()
       .unwrap();

    let data = SomeData { ... };
    let serialized = serialize(&data);
    let send_1 = producer.send_raw("some_topic", serialized, None);
    let send_2 = producer.send_raw("some_topic", serialized, None);
    let send_3 = producer.send_raw("some_topic", serialized, None);

    future::join_all(vec![send_1, send_2, send_3]).wait().unwrap();
    runtime.shutdown_now().wait().unwrap();
}

```
#### Consuming
```rust
use tokio::runtime::Runtime;
use futures::future::Future;
use futures::stream::Stream;
use pulsar::{Ack, ConsumerBuilder, ConsumerError, SubType};

pub struct SomeData {
    ...
}

fn deserialize(data: &[u8]) -> Result<SomeData, Error> {
    ...
}

fn main() {
    let pulsar_addr = "...";
    let runtime = Runtime::new().unwrap();

    let consumer = ConsumerBuilder::new(pulsar_addr, runtime.executor())
        .with_topic("some_topic")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("some_subscription_name")
        .with_deserializer(|payload| deserialize(&payload.data))
        .build()
        .wait()
        .unwrap();

    let consumption_result = consumer
        .for_each(|(msg, ack): (Result<SomeData, Error>, Ack)| match msg {
            Ok(data) => {
                // process data
                ack.ack();
                Ok(())
            },
            Err(e) => {
                println!("Got an error: {}", e);
                Ok(())
                // return Err(_) to instead shutdown consumer
            }
        })
        .wait();

    // handle error, reconnect, etc

    runtime.shutdown_now().wait().unwrap();
}
```
### Connection Pooling
```rust
use r2d2_pulsar::ProducerConnectionManager;

let addr = "127.0.0.1:6650";
let runtime = Runtime::new().unwrap();

let pool = r2d2::Pool::new(ProducerConnectionManager::new(
    addr,
    "r2d2_test_producer",
    runtime.executor()
)).unwrap();

let mut a = pool.get().unwrap();
let mut b = pool.get().unwrap();

let data1: Vec<u8> = ...;
let data2: Vec<u8> = ...;

let send_1 = a.send_raw("some_topic", data1);
let send_2 = b.send_raw("some_topic", data2);

send_1.join(send_2).wait().unwrap();

runtime.shutdown_now().wait().unwrap();
```
### Serde
```rust

#[derive(Debug, Serialize, Deserialize)]
struct SomeData {
    ...
}

fn process_data(data: Result<SomeData, ConsumerError>, ack: Ack) -> Result<(), ConsumerError> {
    ...
}

fn main() {
    let pulsar_addr = "...";
    let runtime = Runtime::new().unwrap();

    let producer = Producer::new(pulsar_addr, None, None, None, runtime.executor())
        .wait()
        .unwrap();

    let consumer = ConsumerBuilder::new(pulsar_addr, runtime.executor())
        .with_topic("some_topic")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("some_subscription_name")
        .build()
        .wait()
        .unwrap();

    let send_1 = producer.send_json("some_topic", &SomeData { ... }, None);
    let send_2 = producer.send_json("some_topic", &SomeData { ... }, None);
    let send_3 = producer.send_json("some_topic", &SomeData { ... }, None);

    future::join_all(vec![send_1, send_2, send_3]).wait().unwrap();

    let consumption_result = consumer
        .for_each(|(msg, ack): (Result<SomeData, ConsumerError>, Ack)| process_data(msg, ack))
        .wait();

    runtime.shutdown_now().wait().unwrap();
}

```

### License
This library is licensed under the terms of both the MIT license and the Apache License (Version 2.0), and may include packages written by third parties which carry their own copyright notices and license terms.

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
