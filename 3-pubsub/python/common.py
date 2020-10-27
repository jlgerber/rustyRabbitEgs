import pika, sys, os
QUEUE = 'somequeue' # shouldnt be used
DURABLE = True
EXCHANGE = 'logs'
EXCHANGE_TYPE = 'fanout'
EXCLUSIVE = True # delete queue when done with it
# We dont care about the routing key as 
# it is ignored in a fanout exchange
ROUTING_KEY = '' 

def connect(queue, host='localhost', durable=False):
    print ("attempting to connect to host: '{}'".format(host))
    connection = pika.BlockingConnection(pika.ConnectionParameters(host))
    channel = connection.channel()
    exchange = channel.exchange_declare(exchange=EXCHANGE, exchange_type=EXCHANGE_TYPE)
    # we have a named exchange and routing key. we dont need
    # a named qeue, so we leave the name blank. RabbitMq will
    # generate a name for us
    queue = channel.queue_declare(
        queue='' if EXCLUSIVE is True else QUEUE, 
        exclusive=EXCLUSIVE)
    queue_name = queue.method.queue
    channel.queue_bind(exchange=EXCHANGE, queue=queue_name)
    
    return (channel, connection, queue_name)