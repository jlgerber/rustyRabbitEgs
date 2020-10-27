#!/usr/bin/env python3

import pika, sys, os
from common import connect

def callback(ch, method, properties, body):
    print(" [X] Received %r" % body)


def consume(channel,queue, callback):
    channel.basic_consume(queue=queue, 
                          on_message_callback=callback,
                          auto_ack=True)

    

def main():
    print("main entered")
    queue = 'hello'
    channel, _connection = connect(queue=queue)
    consume(channel=channel, queue=queue, callback=callback)
    
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