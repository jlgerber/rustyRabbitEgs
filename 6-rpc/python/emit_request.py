#!/usr/bin/env python3

import pika, sys, pickle, pprint
import uuid

from common import (connect, 
                    QUEUE, 
                    DURABLE, 
                    EXCHANGE, 
                    validate_topic_key,
                    RoutingKey)


class FibRpcClient(object):
   
    def __init__(self, host='localhost'):
        """Sets up rpc pattern
         
         - creates connection
         - creates channel
         - creates anonymous, exclusive queue which will be handed 
           to the server as part of the callback. Sice it is exclusive
           it will be destroyed when finished
        """
        self.connection = pika.BlockingConnection(
            pika.ConnectionParameters(host=host)
        )
        self.channel = self.connection.channel()
        self.response = None
        result = self.channel.queue_declare(queue='', exclusive=True)
        self.callback_queue = result.method.queue
        
        self.channel.basic_consume(
            queue=self.callback_queue,
            on_message_callback=self.on_response,
            auto_ack=True
        )
        self.corr_id = None
    
    def on_response(self, ch, method, props, body):
        """Callback which handles data comming back over the queue.
        """
        if self.corr_id == props.correlation_id:
            self.response = pickle.loads(body)
    
    def __call__(self, num):
        """post request to service and wait for response.
        :param (int) num: request the result of calculating the
        fibonnacci series num times. This will return the highest 
        value.
        :returns: fibinacci results as int
        """
        #num = int(num)
        self.corr_id = str(uuid.uuid4())
        self.channel.basic_publish(
            exchange='',
            routing_key=QUEUE,
            properties=pika.BasicProperties(
                reply_to=self.callback_queue,
                correlation_id=self.corr_id
            ),
            body=str(num)
        )
        while self.response is None:
            self.connection.process_data_events()
        if not 'result' in self.response:
            raise RuntimeError("Malformed response: {}".format(pprint.pformat(self.response)))
        return self.response

def format_exc(err):
    pieces = ["\n\t"+x.strip() for x in err.strip().split('\n')]
    rval = "".join(pieces[:-1])
    return rval
    
def main():
    if len(sys.argv) !=2:
        sys.stderr.write("Usage: {} <int>".format(sys.argv[0]))
        sys.exit(1)
    
    fib_rpc = FibRpcClient()
    print("[X] Requesting fib({})".format(sys.argv[1]))
    response = fib_rpc(sys.argv[1])
    if isinstance(response['result'], Exception):
        print("[E] Error\n\t{}\n\n\tCorrelation Id: {}\n\n\t{}: {}".format(
                                                             format_exc(response['traceback']),
                                                             response['correlation_id'],
                                                             response['result'].__class__.__name__,
                                                             str(response['result'])))
        sys.exit(1)
    else:
        print("[.] Result id:{} value:{}".format(response['correlation_id'], response['result']))
    

if __name__ == '__main__':
    try:
        main()
    except Exception as err:
        print ("\n\t{}: {}".format(err.__class__.__name__, err))