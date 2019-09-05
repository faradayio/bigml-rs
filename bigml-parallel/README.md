# `bigml-parallel`: A CLI tool for running WhizzML scripts in parallel

This is a tool for parallel processing of BigML resources (typically sources or datasets) using WhizzML scripts. It's still somewhat experimental. To install, download binaries from the [releases page](https://github.com/faradayio/bigml-rs/releases).

This tool will output the resulting BigML execution objects as JSON structs, one per line, in no particular order. It runs up to `--max-tasks` BigML executions at a time.

```txt
Execute WhizzML script in parallel over one or more BigML resources

USAGE:
    bigml-parallel [OPTIONS] --script <script>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <inputs>...
            Extra inputs to our WhizzML script, specified as "name=value".
            These will be parsed as JSON if possible, or treated as strings
            otherwise.
    -J, --max-tasks <max_tasks>
            How many BigML tasks should we use at a time? [default: 2]

    -n, --name <name>
            The name to use for our execution objects.

    -o, --output <outputs>...
            Expected outputs to our WhizzML script, specified as "name".

    -R, --resource-input-name <resource_input_name>
            The input name used to pass the dataset. [default: resource]

    -r, --resource <resources>...
            The resource IDs to process. (Alternatively, pipe resource IDs on
            standard input, one per line.)
    -s, --script <script>
            The WhizzML script ID to run.
```