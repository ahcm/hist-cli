extern crate plotters;
extern crate serde;
extern crate csv;
use plotters::prelude::*;
use serde::Deserialize;
use std::io;
use std::collections::BTreeMap;
use structopt::StructOpt;
use std::path::PathBuf;
use std::error::Error;
use std::fs::File;

#[allow(non_snake_case)]
#[derive(Debug, StructOpt)]
#[structopt(name = "hist", about = "Plots histogram of input", rename_all="verbatim")]
struct Opt
{
    #[structopt(parse(from_os_str))]
    /// optional file with on entry per line [default: STDIN]
    input: Option<PathBuf>,

    #[structopt(parse(from_os_str), long, short, default_value = "histogram.png")]
    /// file to save PNG plot to
    output: PathBuf,

    #[structopt(long, short)]
    /// do not save a PNG plot to a file
    nooutput: bool,

    #[structopt(long, short)]
    /// also plot a textplot to STDOUT
    textplot: bool,

    #[structopt(parse(from_os_str), long, short)]
    /// save counts data to file as TSV, use - for STDOUT
    save: Option<PathBuf>,

    #[structopt(short, long, default_value = "Counts distribution")]
    /// optional title above the plot
    Title: String,

    #[structopt(short, long, default_value = "1280x960")]
    /// the x and y pixel sizes of the output file
    size: String,

    #[structopt(long, default_value = "Rank")]
    /// x-axis label
    xdesc: String,

    #[structopt(long, default_value = "Counts")]
    /// y-axis label
    ydesc: String,
}

#[derive(Debug, Deserialize)]
struct Record
{
    key: String,
}

fn main() -> Result<(), Box<dyn Error>>
{
    let opt = Opt::from_args();

    let input: Box<dyn std::io::Read + 'static> =
        if let Some(path) = &opt.input
        {
            Box::new(File::open(&path).unwrap())
        }
        else
        {
            Box::new(io::stdin())
        };

    let key_counts = 
        csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(input)
        .deserialize::<Record>()
        .fold(BTreeMap::new(), |mut map,rec|
        { 
            let s = rec.unwrap().key; 
            //*map.entry(s).or_insert(0) += 1;
            map.entry(s).and_modify(|e| *e += 1 ).or_insert(1);
            map
        }); 

    if let Some(path) = &opt.save
    {
        save(&key_counts, &path);
    }

    let mut sorted_counts = key_counts.values().fold(Vec::new(), |mut v, x| { v.push(*x); v });
    sorted_counts.sort();

    if opt.textplot
    {
        let x_dim = (sorted_counts.len() as f64 * 1.1) as usize;
        text_plot(&sorted_counts, 160, 80, 0.0, x_dim as f32);
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

fn save(counts : &BTreeMap<String, usize>, path : &std::path::Path)
{
    let mut out: Box<dyn std::io::Write + 'static> =
        if path == std::path::Path::new("-")
        {
            Box::new(io::stdout())
        }
        else
        {
            Box::new(File::create(&path).unwrap())
        };

    for (key, count) in counts
    {
       out.write_fmt(format_args!("{}\t{}\n", key, count)).expect("Write to save file failed");
    }
}

const BLUE : plotters::style::RGBColor = RGBColor(0x2a, 0x71, 0xb0);

fn next_potence(x : f64) -> f64
{
    10f64.powf(((x.log10() * 10f64).ceil()) / 10.0)
}

fn plot_rank(sorted_counts : &Vec<usize>, opt : &Opt) -> Result<(), Box<dyn Error>>
{
    let max = *sorted_counts.last().expect("At lease one entry is needed");
    let y_dim = next_potence(max as f64) as usize;
    let x_dim = (sorted_counts.len() as f64 * 1.1) as usize;

    let (size_x_str, size_y_str) = opt.size.split_once("x").expect("size not in correct format");
    let size_x = size_x_str.parse().expect("Unable to parse size x");
    let size_y = size_y_str.parse().expect("Unable to parse size y");
    let root = BitMapBackend::new(&opt.output, (size_x, size_y)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(70)
        .y_label_area_size(100)
        .margin(20)
        .caption(&opt.Title, ("sans-serif", 40))
        .build_cartesian_2d((0..x_dim).into_segmented(), 0..y_dim)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc(&opt.ydesc)
        .x_desc(&opt.xdesc)
        .label_style(("sans-serif", 20))
        .axis_desc_style(("sans-serif", 24))
        .draw()?;

    chart.draw_series(
        sorted_counts.iter().rev().enumerate().map(|(x,y)|
                                                   {
                                                       let x0 = SegmentValue::Exact(x);
                                                       let x1 = SegmentValue::Exact(x + 1);
                                                       let mut bar = Rectangle::new([(x0, *y as usize), (x1, 0 as usize)], BLUE.filled());
                                                       bar.set_margin(0, 0, 0, 0);
                                                       bar
                                                   })
        )?;


    Ok(())
}

fn text_plot(sorted_counts : &Vec<usize>, width : u32, height : u32, xmin : f32, xmax : f32)
{
    use textplots::{Chart,Plot,Shape};
    Chart::new(width, height, xmin, xmax as f32)
        .lineplot(&Shape::Bars(
                &sorted_counts
                .iter()
                .rev()
                .enumerate()
                .map(|(x,y)| ((x + 1) as f32, *y as f32)).collect::<Vec<(f32,f32)>>()
                )).display();
}
