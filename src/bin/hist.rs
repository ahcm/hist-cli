extern crate plotters;
extern crate serde;
extern crate csv;
use plotters::prelude::*;
use serde::Deserialize;
use std::io;
use std::collections::BTreeMap;
use structopt::StructOpt;
use std::path::PathBuf;
use std::path::Path;
use std::error::Error;

#[derive(Debug, StructOpt)]
#[structopt(name = "hist", about = "Plots histogram of input")]
struct Opt
{
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
    #[structopt(parse(from_os_str), default_value = "histogram.png")]
    output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Record
{
    key: String,
}

const BLUE : plotters::style::RGBColor = RGBColor(0x2a, 0x71, 0xb0);

fn next_potence(x : f64) -> f64
{
    10f64.powf(((x.log10() * 10f64).ceil()) / 10.0)
}

fn main() -> Result<(), Box<dyn Error>>
{
    let opt = Opt::from_args();

    let mut rdr = csv::ReaderBuilder::new().has_headers(false).delimiter(b'\t').from_reader(io::stdin());
    let data =  rdr.deserialize::<Record>();
    let counts = data.fold(BTreeMap::new(),
      |mut map,rec|
      { 
          let s = rec.unwrap().key; 
          //*map.entry(s).or_insert(0) += 1;
          map.entry(s).and_modify(|e| *e += 1 ).or_insert(1);
          map
      } ); 
    plot_rank(counts, &opt.output)
}

fn plot_rank(counts : BTreeMap<String, i32>, out_path : &Path) -> Result<(), Box<dyn Error>>
{
    let max = counts.values().fold(0, |max,v| max.max(*v));
    let y_dim = next_potence(max as f64) as usize;
    let x_dim = (counts.values().len() as f64 * 1.1) as usize;

    let root = BitMapBackend::new(out_path, (1280, 960)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(70)
        .y_label_area_size(100)
        .margin(20)
        .caption("Counts distribution", ("sans-serif", 40))
        .build_cartesian_2d((0..x_dim).into_segmented(), 0..y_dim)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Rank")
        .disable_x_mesh()
        .label_style(("sans-serif", 20))
        .axis_desc_style(("sans-serif", 24))
        .draw()?;

    let mut sorted_counts = counts.values().fold(Vec::new(), |mut v, x| { v.push(*x); v });
    sorted_counts.sort();
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
