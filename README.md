Launchify
---------

Launchify is a tool that makes it super easy to schedule programs to run on
a regular schedule on macOS. As an example, to schedule a program `myprog` to
run ever 5 minutes:

```launchify 5m myprog```

Under the hood, launchify registers your program to run using `launchd`. It
writes a boilerplate config file to `~/Library/LaunchAgents` and sets it up to
write logs to `~/logs/myprog`.

Advanced Usage
==============

To see full CLI options, run `launchify -h`. You can specify the period to run
the program over as (d)ays, (h)ours, (m)inutes or (s)econds. Extra program args
may be specified via the `--args` option and you may override the default
program name via `--name`.