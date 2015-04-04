rQotd
=====
A simple, configurable QOTD server written in Rust.

Configuration
-------------
When run without arguments, rqotd will load configuration from /etc/rqotd.toml. It uses the TOML file format for configuration. Some example configs:

    port = 17
    execute = "fortune"
    args = ["-a"]
    message = ""

or,

    port = 17
    execute = ""
    message = "Hello World"

The first one executes a program, passing it the arguments specified in args. The second one just returns a given message.

If no options are specified, rqotd will bind to port 17 and immediately close any incomming connections.

