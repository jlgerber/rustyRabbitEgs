#!/usr/bin/env python3

import pika, sys, os
from common import connect, QUEUE, DURABLE
import time

def callback(ch, method, properties, body):
    print(" [X] Received %r" % body.decode())
    
    time.sleep(body.count(b'.'))
    
    print(" [x] Done")
    # manual ack
    ch.basic_ack(delivery_tag = method.delivery_tag)


def consume(channel, queue, callback, prefetch_count=1):
    if prefetch_count:
        # this prevents the worker from accepting more than `prefetch_count`
        # tasks at a time. You would typically set this to 1.
        # this prevents blind round robin dispatching
        channel.basic_qos(prefetch_count=prefetch_count)
    channel.basic_consume(queue=queue, 
                          on_message_callback=callback,
                          auto_ack=False) # now we are doing manual ack'ing

    

def main():
    print("main entered")
  
    channel, _connection = connect(queue=QUEUE, 
                                   durable=DURABLE)

    consume(channel=channel, 
            queue=QUEUE, 
            callback=callback)
    
    print(" [*] Waiting for messages. To exit press CTRL+C")
    channel.start_consuming()
    
if __name__ == '__main__':
    
    try:
        main()
    except KeyboardInterrupt:
        print('Interupted')
        try:
            sys.exit(0)
        except SystemExit:
            os._exit(0)