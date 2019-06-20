use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

use simplelog::*;

use embroidery_lib::failure::Error;
use embroidery_lib::format::traits::{PatternLoader, PatternWriter};
use embroidery_lib::prelude::*;

use embroidery_fmt_dst::{DstPatternLoader, DstPatternWriter};
use embroidery_fmt_svg::SvgPatternWriter;

static DST_FILE: &str = "tests/dst/test_data/OSHLogo.dst";
// static DST_FILE: &str = "tests/dst/test_data/Embroidermodder.dst";
// static DST_FILE: &str = "tests/madeirausa.com/goldfish.dst";
static SVG_FILE: &str = "OSHLogo.svg";

fn main() -> Result<(), Error> {
    TermLogger::init(
        LevelFilter::Debug,
        Config {
            time: None,
            target: None,
            location: Some(Level::Error),
            ..Config::default()
        },
    )?;
    let dst = DstPatternLoader {};
    let dst_w = DstPatternWriter {};
    let svg = SvgPatternWriter {};
    let mut orig_reader = BufReader::new(File::open(DST_FILE)?);
    let orig_pattern = dst.read_pattern(&mut orig_reader)?;

    let mut writer = BufWriter::new(File::create(SVG_FILE)?);
    svg.write_pattern(&orig_pattern, &mut writer)?;

    let pattern = test_read_write_pair(&dst, &dst_w, &orig_pattern, 2);

    let mut writer = BufWriter::new(File::create(SVG_FILE.to_owned() + ".dst")?);
    dst_w.write_pattern(&pattern, &mut writer)?;

    Ok(())
}

fn test_read_write_pair(
    loader: &impl PatternLoader,
    writer: &impl PatternWriter,
    orig_pattern: &Pattern,
    iterations: usize,
) -> Pattern {
    let orig_stitches: Vec<&Stitch> = orig_pattern.iter_stitches().collect();
    let orig_attrs: HashSet<&PatternAttribute> = orig_pattern.attributes.iter().collect();
    let mut pattern = orig_pattern.clone();
    for i in 0..iterations {
        let mut buff = Vec::new();
        writer.write_pattern(&pattern, &mut buff).unwrap();
        pattern = loader.read_pattern(&mut buff.as_slice()).unwrap();

        let attrs: HashSet<&PatternAttribute> = orig_pattern.attributes.iter().collect();
        let stitches: Vec<&Stitch> = pattern.iter_stitches().collect();
        if attrs != orig_attrs {
            warn!("Conversion #{} failed due to mismatched attributes", i);
            for attr in attrs.difference(&orig_attrs) {
                warn!("New attribute:     {:?}", attr);
            }
            for attr in orig_attrs.difference(&attrs) {
                warn!("Missing attribute: {:?}", attr);
            }
        }
        for (i, (orig, new)) in orig_stitches.iter().zip(stitches.iter()).enumerate() {
            if orig != new {
                warn!("Stitches differ at stitch {}. Original: {:?}, New: {:?}", i, orig, new);
                break;
            }
        }
        assert_eq!(orig_attrs, attrs);
        assert_eq!(orig_stitches, stitches);
    }
    pattern
}
