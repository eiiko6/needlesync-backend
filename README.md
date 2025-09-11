# NeedleSync Backend

## Setup

Specify the JWT secret with the `JWT_SECRET` environment variable.

## Development

You can run this to setup the DB:

```sh
podman build -t needlesync-db ./db
podman run -d \
          --name needlesync-db \
          -p 5432:5432 \
          needlesync-db
```
