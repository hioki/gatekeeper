# Gatekeeper: A [SOCKS5] Server written in Rust.

[![CircleCI](https://circleci.com/gh/Idein/gatekeeper.svg?style=svg)](https://circleci.com/gh/Idein/gatekeeper)


## Features
### Authentication Method

Any authentication method is not supported.

The client connects to the server is required for sending `X'00'` (`NO AUTHENTICATION REQUIRED`) as a method selection message.

### Command

Only `CONNECT` command is supported.

### Filter

Gatekeeper allow users to restricting connection based on:

- target address
    - ip address (subnet range)
    - domain name (regex matching)
- port number
- protocol (currently, tcp is only supported)


## Install

You can install gatekeeper as an executable (`gatekeeperd`) from git repository.

```
$ cargo install --locked --git https://github.com/Idein/gatekeeper.git
$ gatekeeperd
gatekeeperd
gatekeeper 0.1.0
```


## How to use

When the gatekeeperd installation is complete, you would be able to run the program.

```
$ gatekeeperd
```

You can look see command line options.

```
$ gatekeeperd --help
```

### Filter Rule

By default, gatekeeper accepts all connection requests.
However, it is possible to filter out some requests along with a filtering rule (described above) given an yaml file.
This yaml file follows special format described below.

#### Format

Any filter rule yaml is constructed from a sequence of `RuleEntries`.
Each `RuleEntry` is either `Allow` or `Deny`.

```yaml
---
- Allow:
    ..
- Deny:
    ..
- Deny:
    ..
- Allow:
    ..
..
```

The rule is in the back of this list have higher precedence.
Then the head of rules is treated as default rule, and the rule should be either `allow all connection` or `deny all connection`.

```yaml
- Allow:
    address: Any
    port: Any
    protocol: Any
..
```

Or

```yaml
- Deny:
    address: Any
    port: Any
    protocol: Any
..
```


All `RuleEntry` have 3 fields `address`, `port` and `protocol`.
Value of these fields are either `Any` or `Specif`.
`Any` matches any values, and `Specif` matches a specified value(s).

- `address`

    ```yaml
    # any address
    address: Any
    ```

  `address` is either `IpAddr` or `Domain`.

    ```yaml
    # 192.168.0.1/24
    address:
      Specif:
        IpAddr:
          addr: 192.168.0.1
          mask: 24
    ```

    ```yaml
    # {mail.,}google.{com,co.jp}
    address:
      Specif:
        Domain:
          # regexp pattern
          pattern: "(mail\.)?google.(com|co\.jp)"
    ```


- `port`

    ```yaml
    # any port number
    port: Any
    ```

    ```yaml
    # match only 8080
    port:
      Specif: 8080
    ```

- `protocol`

    ```yaml
    # any protocol
    protocol: Any
    ```

    ```yaml
    # match only tcp
    protocol:
      Specif: Tcp
    ```


#### Examples

- allow all connections

    ```yaml
    ---
    - Allow:
        address: Any
        port: Any
        protocol: Any
    ```

- allow only local subnet (192.168.0.1/16)

    ```yaml
    ---
    .. default deny ..
    - Allow:
        address:
          Specif:
            IpAddr:
              addr: 192.168.0.1
              mask: 16
        port: Any
        protocol: Any
    ```

- block access to facebook.com and youtube.com

    ```yaml
    ---
    .. default allow ..
    - Deny:
        address:
          Specif:
            Domain:
              pattern: "(www\.)?facebook\.com"
        port: Any
        protocol:
          Specif: Tcp
    - Deny:
        address:
          Specif:
            Domain:
              pattern: "(www\.)youtube\.com"
        port: Any
        protocol:
          Specif: Tcp
    ```



[SOCKS5]: ftp://ftp.rfc-editor.org/in-notes/rfc1928.txt "SOCKS Protocol Version 5"
