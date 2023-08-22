# Proxytheus

Proxytheus is a proxy server for Prometheus metrics. It is designed to allow
secure access to Prometheus metrics from applications that are not capable of
authenticating properly.

I mainly built this in order to allow the [Datadog Agent](https://docs.datadoghq.com/containers/kubernetes/prometheus/?tab=kubernetesadv2)
to scrape metrics from a Prometheus server that is secured with OAuth2 or TLS.
Datadog does support authentication with OAuth2, but it does not support passing
custom query parameters, such as the `audience` parameter required for endpoints
secured by Auth0.

## Usage

### Configuration

Proxytheus is configured via either command line arguments or environment
variables. The following table lists the available configuration options:

| Command Line Argument | Environment Variable   | Description                      |
| --------------------- | ---------------------- | -------------------------------- |
| `--help`, `-h`        |                        | Show help message                |
| `--address`, `-a`     | `ADDRESS`              | Address to listen on             |
| `--port`, `-p`        | `PORT`                 | Port to listen on                |
| `--endpoint`, `-e`    | `ENDPOINT`             | Prometheus metrics URL           |
| `--client-id`         | `OAUTH2_CLIENT_ID`     | OAuth2 client ID                 |
| `--client-secret`     | `OAUTH2_CLIENT_SECRET` | OAuth2 client secret             |
| `--auth-url`          | `OAUTH2_AUTH_URL`      | OAuth2 authorization URL         |
| `--token-url`         | `OAUTH2_TOKEN_URL`     | OAuth2 token URL                 |
| `--audience`          | `OAUTH2_AUDIENCE`      | OAuth2 audience                  |
| `--header-name`       | `OAUTH2_HEADER_NAME`   | OAuth2 access token header name  |
| `--header-value`      | `OAUTH2_HEADER_VALUE`  | OAuth2 access token header value |
| `--cert`              | `TLS_CERT`             | TLS certificate contents         |
| `--cert-file`         | `TLS_CERT_FILE`        | TLS certificate filepath         |
| `--key`               | `TLS_KEY`              | TLS key contents                 |
| `--key-file`          | `TLS_KEY_FILE`         | TLS key filepath                 |

A few different forms of authentication are supported. The following table lists the
supported authentication methods and the required configuration options:

| Method | Required                                                 | Optional                                  |
| ------ | -------------------------------------------------------- | ----------------------------------------- |
| None   |                                                          |                                           |
| OAuth2 | `client-id`, `client-secret`, `auth-url`, `token-url`    | `audience`, `header-name`, `header-value` |
| TLS    | `tls-cert`, `tls-key` OR `tls-cert-file`, `tls-key-file` |                                           |

Exactly one authentication method must be configured. You must always specify
the `--metrics-endpoint` option, which is the URL of the Prometheus metrics
endpoint that you want to proxy. If `--address` and `--port` are not specified,
the server listens on `0.0.0.0:3000` by default.

### Deployment

> **Important**
> Because Proxytheus allows any requester to effectively bypass authentication
> with the Prometheus server, it is **strongly recommended** that you:
>
> 1. Only run Proxytheus on a private network that is not accessible from the
>    public internet
> 2. Add an additional authentication layer on top of the Proxytheus server
>
> If you don't do _at least_ one of these things, you are effectively allowing
> anyone to access your Prometheus metrics without restriction.

Proxytheus is available as a Docker image at [`samwaterbury/proxytheus`](https://hub.docker.com/r/samwaterbury/proxytheus).
The docker image can be run with:

```sh
docker run -p 3000:3000 samwaterbury/proxytheus:latest --endpoint <endpoint>
```

It was primarily designed to be deployed as a lightweight Kubernetes pod, but in
principle it should be possible to deploy it anywhere that Docker is supported.
You can also manually run the `proxytheus` binary if you prefer.

### Building

You can build the `proxytheus` binary with:

```sh
cargo build --release
```

It will be located at `target/release/proxytheus`. You can also build the Docker
image with:

```sh
docker build . --tag proxytheus:latest
```
