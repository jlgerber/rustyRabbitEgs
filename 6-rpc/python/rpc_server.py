#!/usr/bin/env python3

# stdlib
import sys, os, time, pickle, traceback

# external libs
import pika

# internal
from common import connect, QUEUE

class Cacheable(object):
    def __init__(self, cache, default_value):
        self._cache_path = cache
        self._cache = None
        self._load_cache(default_value)
    
    def _load_cache(self, default_value):
        try:
            with open(self._cache_path,'r') as fh:
                self._cache = pickle.loads(fh)
        except Exception:
            print ("loading default cache")
            self._cache = default_value
    
    def write_cache(self):
        with open(self._cache_path,'w') as fh:
            pickle.dump(self._cache, fh)
    
    def cache_has(self, key):
        return key in self._cache
    
    def cache_set(self, key, value):
        self._cache[key] = value  
    
    def cache_get(self, key):
        return self._cache[key]
            
class Callback(Cacheable):
    """Callable class which manages interfacing with rabbit, as well as 
    caching of results
    """
    def __init__(self,cache='/var/tmp/__fib_cache__'):
        self._count = 0
        super(Callback, self).__init__(cache, {0:0, 1:1})
    
    def __del__(self):
        self.write_cache()
    
    def fib(self, num):
        if not self.cache_has(num):
            self.cache_set(num, self._fib(num))

        return self.cache_get(num)
    
    def _fib(self, num):
        return self.fib(num-1) + self.fib(num - 2)
    
    def __call__(self, ch, method, props, body):
        self._count +=1
        try:
            n = int(body)
        except ValueError as err:
            response = {'result': err, 
                        'traceback':traceback.format_exc(),
                        'correlation_id': props.correlation_id}
        else:
            print(" [X] %r[%d]: fib(%s)" % (props.reply_to, self._count, n))
            try:
                response = {'result': self.fib(n), 
                            'correlation_id': props.correlation_id }
            except Exception as err:
                response = {'result': err, 
                            'traceback':traceback.format_exc(),
                            'correlation_id': props.correlation_id}
        ch.basic_publish(exchange='', 
                         routing_key=props.reply_to,
                         properties=pika.BasicProperties(correlation_id=props.correlation_id),
                         body=pickle.dumps(response))
        # manual ack
        ch.basic_ack(delivery_tag = method.delivery_tag)
    

def consume(channel, queue, callback):
    channel.basic_qos(prefetch_count=1)
    channel.basic_consume(queue, on_message_callback=callback)
    print(" [x] Awating RPC requests")
    channel.start_consuming()
       
def main():
    
    channel, _connection, queue_name = connect(queue=QUEUE)
                                               
    consume(channel=channel, queue=QUEUE, callback=Callback())
    
    print(" [*] Waiting for logs. To exit press CTRL+C")
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