# RPC
What if we need to run a procedure on a remote computer and wait for the result?
We are going to build a fibonacci server.

For sending data back, we need to set up a queue and a corellation_id in the client, and pass them to the server along with the rest of the request. Then, the server will submit the response on the supplied queue, which the sender will monitor for the response.

![rpc](rpc.png)

# Future questions
- How shoudl the client react if there are no servers running?
- should client have some sort of timeout?