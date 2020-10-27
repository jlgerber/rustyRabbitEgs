#!/usr/bin/env python3

import pika, sys
from common import (connect, 
                    QUEUE, 
                    DURABLE, 
                    EXCHANGE, 
                    RoutingKey)


def send_msg_on_channel(key, msg, channel):
    channel.basic_publish(
        exchange=EXCHANGE, 
        routing_key=key, # our exchange is a 
        body=msg
    )
    print(" [x] Sent '{}'".format(msg))
    

def main():
    if len(sys.argv) < 3:
        sys.stderr.write("Usage: {} [info|warn|error] [msg]".format(sys.argv[1]))
        sys.exit(1)
    try:
        loglevel = RoutingKey.validate(sys.argv[1])
    except KeyError as err:
        sys.stderr.write(err)
        sys.exit(1)
    
    msg = ' '.join(sys.argv[2:]) or "Hello World"
    channel, connection, _queue_name = connect(queue=QUEUE, durable=DURABLE, routing_keys=None)
    send_msg_on_channel(key=loglevel, msg=msg, channel=channel)
    connection.close()
    
if __name__ == '__main__':
    try:
        main()
    except Exception as err:
        print ("ERROR occured: {}".format(err))