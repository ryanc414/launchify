Launchify
---------

Launchify is a tool that makes it super easy to schedule programs to run on
a regular schedule on macOS. As an example, to schedule a program `myprog` to
run ever 5 minutes:

```$ launchify 5m myprog```

Under the hood, launchify registers your program to run using `launchd`. It
writes a boilerplate config file to `~/Library/LaunchAgents` and sets it up to
write logs to `~/logs/myprog`.

Install
-------

Install requires the rust toolchain, which may be installed like:

```$ curl https://sh.rustup.rs -sSf | sh```

Then install using `cargo`:

```$ cargo install launchify```

Advanced Usage
==============

To see full CLI options, run `launchify -h`.

You can specify the period to run the program over as (d)ays, (h)ours, (m)inutes
or (s)econds. For example, to run `myprog` once an hour:

```$ launchify 1h myprog```

Extra program args may be specified via the `--args` option

```$ launchify 5m myprog --args="--foo bar"```

You may override the default name used to label the launchify job and log
directory via `--name`. By default the name is derived from the program filename.

```$ launchify 5m myprog --name=my_awesome_program```