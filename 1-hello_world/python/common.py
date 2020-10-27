import pika, sys, os

def connect(queue,host='localhost'):
    print ("attempting to connect to host: '{}'".format(host))
    connection = pika.BlockingConnection(pika.ConnectionParameters(host))
    channel = connection.channel()
    channel.queue_declare(queue=queue)
    return (channel, connection)