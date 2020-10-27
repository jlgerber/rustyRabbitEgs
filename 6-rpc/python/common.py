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
        

QUEUE = 'rpc_queue' # shouldnt be used
DURABLE = True
EXCHANGE = ''
EXCHANGE_TYPE = ExchangeType.Direct 
EXCLUSIVE = True # delete queue when done with it
# We dont care about the routing key as 
# it is ignored in a fanout exchange


def connect(queue=QUEUE, 
            host='localhost'):
    """Set up the connection, channel, and exchange (if non-extant)
    
    :param (str) queue: The name of the queue
    "param (str) host: name of host
    """
    print ("attempting to connect to host: '{}'".format(host))
    connection = pika.BlockingConnection(pika.ConnectionParameters(host=host))
    channel = connection.channel()
    queue = channel.queue_declare(queue=queue)
    
    queue_name = queue.method.queue
    return (channel, connection, queue_name)

    
def bind(channel, queue_name, exchange=EXCHANGE, routing_keys=None):
    if routing_keys:
        if len(routing_keys) > 0:
            for key in routing_keys:
                channel.queue_bind(exchange=exchange, 
                                queue=queue_name,
                                routing_key=key)
        else:
            channel.queue_bind(exchange=exchange, 
                            queue=queue_name)   
    

