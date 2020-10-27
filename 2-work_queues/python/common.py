import pika, sys, os
QUEUE = 'task_queue'
DURABLE = True

def connect(queue, host='localhost', durable=False):
    print ("attempting to connect to host: '{}'".format(host))
    connection = pika.BlockingConnection(pika.ConnectionParameters(host))
    channel = connection.channel()
    channel.queue_declare(queue=queue, durable=durable)
    return (channel, connection)