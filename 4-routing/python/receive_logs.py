#!/usr/bin/env python3

import pika, sys, os
from common import connect, QUEUE, DURABLE, RoutingKey
import time

def callback(ch, method, properties, body):
    print(" [X] %r:%r" % (method.routing_key.upper(),body.decode()))
    
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
    # routing keys
    usage = "Usage: {} [info] [warning] [error]\n".format(sys.argv[0])
    if len(sys.argv)<2:
        sys.stderr.write(usage)
        sys.exit(1)
        
    log_levels = sys.argv[1:]
    
    if not log_levels:
        sys.stderr.write(usage)
        sys.exit(1)
    try:
        for level in log_levels:
            RoutingKey.validate(level)
    except KeyError as e:
        sys.stderr.write("Not a valid key: {}\n".format(level))
        sys.stderr.write(usage)
        sys.exit(1)
        
    channel, _connection, queue_name = connect(queue=QUEUE, 
                                               durable=DURABLE,
                                               routing_keys=log_levels)
    
    consume(channel=channel, queue=queue_name, callback=callback)
    
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