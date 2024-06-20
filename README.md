Basic program to forward syslog (/dev/log) to stdout.

USAGE: ./syslog2stdout /dev/log

Example containerized usage:

```
docker run --rm -v dev-log:/shared mcluseau/syslog2stdout /shared/log
docker run --rm -v dev-log:/shared debian -c "ln -s /shared/log /dev/log && /path/to/legacy/daemon"
```
