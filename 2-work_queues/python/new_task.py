#!/usr/bin/env python3

import pika, sys
from common import connect, QUEUE, DURABLE

def send_msg_on_channel(msg, channel):
    channel.basic_publish(
        exchange='', 
        routing_key=QUEUE,
        body=msg
    )
    print(" [x] Sent '{}'".format(msg))
    

def main():
    msg = ' '.join(sys.argv[1:]) or "Hello World"
    channel, connection = connect(queue=QUEUE, durable=DURABLE)
    send_msg_on_channel(msg=msg, channel=channel)
    connection.close()
    
if __name__ == '__main__':
    try:
        main()
    except Exception as err:
        print ("ERROR occured: {}".format(err))