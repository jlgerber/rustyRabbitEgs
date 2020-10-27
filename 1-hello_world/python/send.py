#!/usr/bin/env python3

import pika
from common import connect

def send_msg_on_channel(msg, channel):
    channel.basic_publish(
        exchange='', 
        routing_key='hello',
        body=msg
    )
    print(" [x] Sent '{}'".format(msg))
    

def main():
    channel, connection = connect(queue="hello")
    send_msg_on_channel("hello world", channel)
    connection.close()
    
if __name__ == '__main__':
    try:
        main()
    except Exception as err:
        print ("ERROR occured: {}".format(err))