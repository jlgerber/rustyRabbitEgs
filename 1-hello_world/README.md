# Hello world.
The simplest example. 
A consumer binds directly to a named queue
a producer sends to that queue
![helloworld](./helloworld.png)

## Setup Steps
- establish a connection to the rabbit host
- construct a channel from the connection
- declare a named queue via the channel