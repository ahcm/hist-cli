# hist
Commandline tool for plotting frequency ranked histograms of TSV/CSV data.
## Installation
```
$ cargo install hist-cli
```

## Usage
```
$ hist --help
hist 0.3.0
Plots histogram of input

USAGE:
    hist [OPTIONS] [input]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <output>     [default: histogram.png]
    -s, --size <size>        the x and y pixel sizes of the output file [default: 1280x960]
    -t, --title <title>      optional title above the plot [default: Counts distribution]
        --xdesc <xdesc>      x-axis label [default: Rank]
        --ydesc <ydesc>      y-axis label [default: Counts]

ARGS:
    <input>    
```

Just piping from stdin:
```
$ hist < data.tsv
$ open histogram.png # on MacOS, on Linux maybe display or eog
```


![histogram](https://github.com/ahcm/hist-cli/blob/main/doc/histogram.png)

