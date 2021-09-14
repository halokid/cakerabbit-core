
import os
import time
import mprpc


pid = os.getpid()
print(f'{pid}: {time.time()}: client connecting')
client = mprpc.RPCClient('127.0.0.1', 9527)
print(f'{pid}: {time.time()}: client connected, sending request')
rsp = client.call('say_hello', "foo")
print(f'{pid}: {time.time()}: got response => {rsp}')

