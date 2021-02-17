Launchify
=========

Launchify is a tool that makes it super easy to schedule programs to run as
regular background tasks on macOS. As an example, to schedule a program `myprog`
to run ever 5 minutes:

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
--------------

To see full CLI options, run `launchify -h`.

You can specify the period to run the program over as (d)ays, (h)ours, (m)inutes
or (s)econds. For example, to run `myprog` once an hour:

```$ launchify 1h myprog```

Note that `myprog` may be either the absolute or relative path to an executable,
or the name of an executable on your PATH.

Extra program args may be specified via the `--args` option

```$ launchify 5m myprog --args="--foo bar"```

You may override the default name used to label the launchify job and log
directory via `--name`. By default the name is derived from the program filename.

```$ launchify 5m myprog --name=my_awesome_program```

By default, the task will be configured to run in the same working directory
that you run `launchify` inside, however this may be overridden via the
`--working-dir` option:

```$ launchify 5m myprog --working-dir=/path/to/dir```

Comparison to launchctl
-----------------------

`launchify` is not intended to replace `launchctl` but is a convenience tool
to complement it. `launchify` purposely does not support the full configuration
options which may be passed to `launchctl` but optimizes for a common use-case.

After scheduling a program using `launchify`, you will find the configuration
file written to `~/Library/LaunchAgents/com.<name>.plist`. To stop running
the program, run:

```$ launchctl unload ~/Library/LaunchAgents/com.<name>.plist```

For further information on launch agents and daemones on macOS, see
https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html
