#!/usr/bin/env python3

import pika, sys, os
from common import (bind, 
                    connect, 
                    validate_topic_binding_key,
                    QUEUE, 
                    DURABLE, 
                    RoutingKey)
import time
COUNT=0
class Callback(object):
    def __init__(self):
        self._count = 0
        
    def __call__(self, ch, method, properties, body):
        self._count +=1
        print(" [X] %r[%d]:%r" % (method.routing_key.upper(),self._count,body.decode()))
        
        time.sleep(body.count(b'.'))
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
    # routing keys
    usage = "Usage: {} [binding_key]...\n".format(sys.argv[0])
    if len(sys.argv)<2:
        sys.stderr.write(usage)
        sys.exit(1)
        
    binding_keys = sys.argv[1:]
    
    if not binding_keys:
        sys.stderr.write(usage)
        sys.exit(1)
    try:
        for bkey in binding_keys:
            validate_topic_binding_key(bkey)
    except KeyError as e:
        sys.stderr.write("Not a valid key: {}\n".format(bkey))
        sys.stderr.write(usage)
        sys.exit(1)
        
    channel, _connection, queue_name = connect(queue=QUEUE, 
                                               durable=DURABLE)
                                               
    bind(channel, queue_name=queue_name, routing_keys=binding_keys)
    
    consume(channel=channel, queue=queue_name, callback=Callback())
    
    print(" [*] Waiting for logs. To exit press CTRL+C")
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