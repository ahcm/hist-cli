use clap::Parser;
use csv::StringRecord;
use plotters::prelude::*;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug)]
enum HistError
{
    Io(io::Error),
    Csv(csv::Error),
    Parse(String),
    Validation(String),
}

impl fmt::Display for HistError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            HistError::Io(e) => write!(f, "IO error: {}", e),
            HistError::Csv(e) => write!(f, "CSV error: {}", e),
            HistError::Parse(e) => write!(f, "Parse error: {}", e),
            HistError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for HistError {}

impl From<io::Error> for HistError
{
    fn from(e: io::Error) -> Self
    {
        HistError::Io(e)
    }
}

impl From<csv::Error> for HistError
{
    fn from(e: csv::Error) -> Self
    {
        HistError::Csv(e)
    }
}

type Result<T> = std::result::Result<T, HistError>;

#[derive(Debug, Parser)]
#[command(name = "hist", about = "Plots histogram of input", version)]
struct Opt
{
    /// optional file with on entry per line [default: STDIN]
    input: Option<PathBuf>,

    #[arg(short, long, default_value = "\t")]
    /// column delimiter (supports: \t, tab, comma, space, semicolon)
    delimiter: String,

    #[arg(long, short, default_value = "1")]
    /// key (column) selector (1-indexed)
    key: usize,

    #[arg(long, short, default_value = "histogram.png")]
    /// file to save PNG plot to
    output: PathBuf,

    #[arg(long, short)]
    /// do not save a PNG plot to a file
    nooutput: bool,

    #[arg(long, short = 'H')]
    /// input has header line (see also --skip)
    header: bool,

    #[arg(long, short)]
    /// also plot a textplot to STDOUT
    textplot: bool,

    #[arg(long, short)]
    /// save counts data to file as TSV, use - for STDOUT
    save: Option<PathBuf>,

    #[arg(short = 'T', long, default_value = "Counts distribution")]
    /// optional title above the plot
    title: String,

    #[arg(short, long, default_value = "1280x960")]
    /// the x and y size of the plot (format: WIDTHxHEIGHT)
    geometry: String,

    #[arg(long, default_value = "Rank")]
    /// x-axis label
    xdesc: String,

    #[arg(long, default_value = "Counts")]
    /// y-axis label
    ydesc: String,
}

fn main() -> Result<()>
{
    let opt = Opt::parse();

    validate_options(&opt)?;

    let input: Box<dyn std::io::Read + 'static> = if let Some(path) = &opt.input
    {
        Box::new(File::open(path)?)
    }
    else
    {
        Box::new(io::stdin())
    };

    let delimiter = parse_delimiter(&opt.delimiter)?;

    if opt.key == 0
    {
        return Err(HistError::Validation("Key must be 1 or greater".to_string()));
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(opt.header)
        .delimiter(delimiter)
        .from_reader(input);

    let mut key_counts = BTreeMap::new();
    let mut record = StringRecord::new();
    while reader.read_record(&mut record)?
    {
        let field = record.get(opt.key - 1).ok_or_else(|| {
            HistError::Validation(format!("Column {} not found in record", opt.key))
        })?;
        let s = field.to_string();
        key_counts.entry(s).and_modify(|e| *e += 1).or_insert(1);
    }

    if key_counts.is_empty()
    {
        return Err(HistError::Validation("No data found to plot".to_string()));
    }

    if let Some(path) = &opt.save
    {
        save(&key_counts, path)?;
    }

    let mut sorted_counts = Vec::with_capacity(key_counts.len());
    for count in key_counts.values()
    {
        sorted_counts.push(*count);
    }
    sorted_counts.sort();

    if opt.textplot
    {
        let x_dim = (sorted_counts.len() as f64 * 1.1) as usize;
        text_plot(&sorted_counts, 160, 80, 0.0, x_dim as f32)?;
    }

    if opt.nooutput
    {
        Ok(())
    }
    else
    {
        plot_rank(&sorted_counts, &opt)
    }
}

/// Validates command line options
fn validate_options(opt: &Opt) -> Result<()>
{
    // Validate geometry format
    if !opt.geometry.contains('x') || opt.geometry.matches('x').count() != 1
    {
        return Err(HistError::Validation("Geometry must be in format WIDTHxHEIGHT".to_string()));
    }

    let (width_str, height_str) = opt
        .geometry
        .split_once('x')
        .ok_or_else(|| HistError::Validation("Invalid geometry format".to_string()))?;

    width_str
        .parse::<u32>()
        .map_err(|_| HistError::Parse("Invalid width in geometry".to_string()))?;
    height_str
        .parse::<u32>()
        .map_err(|_| HistError::Parse("Invalid height in geometry".to_string()))?;

    Ok(())
}

/// Parses delimiter string into byte delimiter
fn parse_delimiter(delimiter_str: &str) -> Result<u8>
{
    match delimiter_str
    {
        "\\t" | "tab" | "TAB" => Ok(b'\t'),
        "comma" | "," => Ok(b','),
        "space" | " " => Ok(b' '),
        "semicolon" | ";" => Ok(b';'),
        s =>
        {
            let bytes = s.as_bytes();
            if bytes.is_empty()
            {
                Err(HistError::Validation("Delimiter cannot be empty".to_string()))
            }
            else
            {
                Ok(bytes[0])
            }
        }
    }
}

/// Saves counts data to file or stdout
fn save(counts: &BTreeMap<String, usize>, path: &std::path::Path) -> Result<()>
{
    let mut out: Box<dyn Write + 'static> = if path == std::path::Path::new("-")
    {
        Box::new(io::stdout())
    }
    else
    {
        Box::new(File::create(path)?)
    };

    let mut entries = Vec::from_iter(counts);
    entries.sort_by(|&(_, a), &(_, b)| a.cmp(&b)); // sort by value

    for (key, count) in entries
    {
        writeln!(out, "{}\t{}", count, key)?;
    }

    Ok(())
}

const BLUE: plotters::style::RGBColor = RGBColor(0x2a, 0x71, 0xb0);

fn next_potence(x: f64) -> f64
{
    10f64.powf(((x.log10() * 10f64).ceil()) / 10.0)
}

/// Creates a PNG plot of the histogram data
fn plot_rank(sorted_counts: &Vec<usize>, opt: &Opt) -> Result<()>
{
    let max = *sorted_counts
        .last()
        .ok_or_else(|| HistError::Validation("At least one entry is needed".to_string()))?;
    let y_dim = next_potence(max as f64) as usize;
    let x_dim = (sorted_counts.len() as f64 * 1.1) as usize;

    let (geometry_x_str, geometry_y_str) = opt
        .geometry
        .split_once("x")
        .ok_or_else(|| HistError::Parse("Geometry not in correct format".to_string()))?;
    let geometry_x: u32 = geometry_x_str
        .parse()
        .map_err(|_| HistError::Parse("Unable to parse geometry width".to_string()))?;
    let geometry_y: u32 = geometry_y_str
        .parse()
        .map_err(|_| HistError::Parse("Unable to parse geometry height".to_string()))?;

    let root = BitMapBackend::new(&opt.output, (geometry_x, geometry_y)).into_drawing_area();

    root.fill(&WHITE)
        .map_err(|e| HistError::Parse(format!("Failed to fill background: {}", e)))?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(70)
        .y_label_area_size(100)
        .margin(20)
        .caption(&opt.title, ("sans-serif", 40))
        .build_cartesian_2d((0..x_dim).into_segmented(), 0..y_dim)
        .map_err(|e| HistError::Parse(format!("Failed to build chart: {}", e)))?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc(&opt.ydesc)
        .x_desc(&opt.xdesc)
        .label_style(("sans-serif", 20))
        .axis_desc_style(("sans-serif", 24))
        .draw()
        .map_err(|e| HistError::Parse(format!("Failed to draw chart mesh: {}", e)))?;

    chart
        .draw_series(sorted_counts.iter().rev().enumerate().map(|(x, y)| {
            let x0 = SegmentValue::Exact(x);
            let x1 = SegmentValue::Exact(x + 1);
            let mut bar = Rectangle::new([(x0, *y as usize), (x1, 0 as usize)], BLUE.filled());
            bar.set_margin(0, 0, 0, 0);
            bar
        }))
        .map_err(|e| HistError::Parse(format!("Failed to draw chart series: {}", e)))?;

    Ok(())
}

/// Displays a text-based plot to stdout
fn text_plot(
    sorted_counts: &Vec<usize>,
    width: u32,
    height: u32,
    xmin: f32,
    xmax: f32,
) -> Result<()>
{
    use textplots::{Chart, Plot, Shape};
    let max = *sorted_counts
        .last()
        .ok_or_else(|| HistError::Validation("At least one entry is needed".to_string()))?;
    let y_dim = next_potence(max as f64) as f32;

    let data: Vec<(f32, f32)> = sorted_counts
        .iter()
        .rev()
        .enumerate()
        .map(|(x, y)| ((x + 1) as f32, *y as f32))
        .collect();

    Chart::new_with_y_range(width, height, xmin, xmax, 0.0, y_dim)
        .lineplot(&Shape::Bars(&data))
        .display();

    Ok(())
}
