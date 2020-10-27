import pika, sys, os
QUEUE = 'somequeue' # shouldnt be used
DURABLE = True
EXCHANGE = 'logs_direct'
EXCHANGE_TYPE = 'direct'
EXCLUSIVE = True # delete queue when done with it
# We dont care about the routing key as 
# it is ignored in a fanout exchange
class RoutingKey(object):
    """enumerate the valid routing keys
    and provide validation"""
    Info="info"
    Warn="warn"
    Error="error"
     
    @classmethod
    def validate(cls, key):
        if key == cls.Info or \
        key == cls.Warn or \
        key == cls.Error:
            return key
        raise KeyError("{} is not a valid routing key".format(key))
        

def connect(queue, host='localhost', durable=False, routing_keys=None):
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
    if routing_keys and len(routing_keys) > 0:
        for key in routing_keys:
            channel.queue_bind(exchange=EXCHANGE, 
                            queue=queue_name,
                            routing_key=key)
    else:
        channel.queue_bind(exchange=EXCHANGE, 
                           queue=queue_name)   
    
    return (channel, connection, queue_name)