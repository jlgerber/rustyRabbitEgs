# rustyRabbitEgs

These examples rely on a local docker instance of rabbitmq running. To run this, do the following:

```bash
sudo docker run -d --hostname my-rabbit --name some-rabbit -p 8080:15672 -p 5672:5672 rabbitmq:3-management
```
This command forwards the rabbit port on your localhost to the container. it also forwards port 8080 on your local to the docker container's management webserver's port. 