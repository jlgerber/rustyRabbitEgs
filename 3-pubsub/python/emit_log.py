#!/usr/bin/env python3

import pika, sys
from common import connect, QUEUE, DURABLE, EXCHANGE, ROUTING_KEY

def send_msg_on_channel(msg, channel):
    channel.basic_publish(
        exchange=EXCHANGE, 
        routing_key=ROUTING_KEY, # our exchange is a 
        body=msg
    )
    print(" [x] Sent '{}'".format(msg))
    

def main():
    msg = ' '.join(sys.argv[1:]) or "Hello World"
    channel, connection, _queue_name = connect(queue=QUEUE, durable=DURABLE)
    send_msg_on_channel(msg=msg, channel=channel)
    connection.close()
    
if __name__ == '__main__':
    try:
        main()
    except Exception as err:
        print ("ERROR occured: {}".format(err))