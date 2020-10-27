# Hello World
This example illustrates the simplest topology possible, consisting of three components:
- producer - a program which sends messages
- queue - an object which is created by the RabbitMq runtime on the RabbitMq server, bound only by the host's memory and disk limits. Many producers can send messages to one queue, and many consumers can try to receive data from one queue.
- consumer - a program that processes messages in the queue in fifo order.

The producer, consumer, and broker do not have to reside on the same host.

## Using lapin
For this tutorial, we are using the lapin client library for Rust. Our aim is to model the following topology:
![hello](../helloworld.png)
