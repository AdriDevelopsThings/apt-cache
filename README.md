# apt-cache
A simple apt repository cache

## Installation
Run apt-cache by building it yourself with `cargo` or using docker:
```
docker run --name apt-cache -v ./cache:/cache -v ./config.toml:/config.toml -p 80:80 -d ghcr.io/adridevelopsthings/apt-cache:main
```

## Configuration
The `config.toml` should look like this
```toml
cache_directory = "cache" # not required, this is the default value
cache_ttl = 1440 # not required, this is the default value, ttl in minutes
disable_logging = false # not required, this is the default value

# some examples for repositories
[[repository]]
name = "ubuntu-ports"
url = "http://ports.ubuntu.com/ubuntu-ports/"

[[repository]]
name = "ubuntu-archive"
url = "http://de.archive.ubuntu.com/ubuntu/"
```

## How to use now?

Just replace for example `http://ports.ubuntu.com/ubuntu-ports/` with `http://apt_cache_ip/ubuntu-ports/`. The first part of the path of apt-cache must be the repostiroy name. Apt-cache will resolve the name to the url to make the request to the repository.

## Environment variables
| Variable         | Description                                           | Default                               |
| ---------------- | ----------------------------------------------------- | ------------------------------------- |
| `LISTEN_ADDRESS` | The address where apt-cache should listen             | `127.0.0.1:8000`, docker=`0.0.0.0:80` |
| `CACHE_PATH`     | Override the `cache_directory` from the `config.toml` | `cache`, docker=`/cache`              |
| `CONFIG_PATH`    | The path where the config file is (TOML)              | `config.toml`, docker=`/config.toml`  |
