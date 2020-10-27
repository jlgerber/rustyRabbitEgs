import pika, sys, os
import re 

bkre = re.compile(r'^(([#*]{0,1}[a-zA-Z0-9]+|[a-zA-Z0-9]+[#*]{0,1}|[#*]{0,1})*\.)*([#*]{0,1}[a-zA-Z0-9]+|[a-zA-Z0-9]+[#*]{0,1}|[#*]{0,1})$')
tkre = re.compile(r'^(([a-zA-Z0-9]\.)*[a-zA-Z0-9])+$')

def validate_topic_binding_key(key):
    if bkre.match(key):
        return key
    raise KeyError("{} is not a valid key.".format(key))


def validate_topic_key(key):
    if tkre.match(key):
        return key
    else:
        raise KeyError("{} is not a valid routing key".format(key))
    

class ExchangeType(object):
    Fanout='fanout'
    Direct='direct'
    Topic='topic'
    

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
        

QUEUE = '' # shouldnt be used
DURABLE = True
EXCHANGE = 'logs_topic'
EXCHANGE_TYPE = ExchangeType.Topic 
EXCLUSIVE = True # delete queue when done with it
# We dont care about the routing key as 
# it is ignored in a fanout exchange


def connect(queue=QUEUE, 
            host='localhost',
            exchange_type=EXCHANGE_TYPE,
            exchange=EXCHANGE, 
            durable=False,
            exclusive=True):
    """Set up the connection, channel, and exchange (if non-extant)
    
    :param (str) queue: The name of the queue
    :param (ExchangeType) exchange_type: The type of exchange
    :param (bool) durable: Whether the queue is persisted to disk
    :param (bool) exclusive: delete queue when finished with it
    """
    print ("attempting to connect to host: '{}'".format(host))
    connection = pika.BlockingConnection(pika.ConnectionParameters(host))
    channel = connection.channel()
    exchange = channel.exchange_declare(exchange=exchange, exchange_type=exchange_type)
    # we have a named exchange and routing key. we dont need
    # a named qeue, so we leave the name blank. RabbitMq will
    # generate a name for us
    queue = channel.queue_declare(
        queue='' if exclusive is True else queue, 
        exclusive=exclusive)
    
    queue_name = queue.method.queue
    
    return (channel, connection, queue_name)

    
# def bind(channel, queue_name, exchange=EXCHANGE, routing_keys=None):
#     if routing_keys:
#         if len(routing_keys) > 0:
#             for key in routing_keys:
#                 channel.queue_bind(exchange=exchange, 
#                                 queue=queue_name,
#                                 routing_key=key)
#         else:
#             channel.queue_bind(exchange=exchange, 
#                             queue=queue_name)   
    

