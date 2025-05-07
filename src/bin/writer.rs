use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use svg::node::element::{path::Data as SvgData, Path as SvgPath};

use bevy_sdf_text::{printable_ascii_chars, SdfAtlasParams, SdfFont};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The input font files
    #[arg(value_name = "FONT_FILE")]
    input: Vec<PathBuf>,

    /// The name of target font
    #[arg(long)]
    name: Option<String>,

    /// The output directory. If not specified, use base directory of input file
    #[arg(short, long)]
    out_dir: Option<PathBuf>,

    /// If true, dump atlas image as `<BASE_NAME>-atlas.png`
    #[arg(long, default_value_t = false)]
    dump_atlas: bool,

    /// If true, dump preview image as `<BASE_NAME>-preview.png`
    #[arg(long)]
    dump_preview: bool,
}

pub fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();

    if cli.input.len() > 1 && cli.name.is_some() {
        return Err(anyhow!(
            "Single name should not be used for multiple input files"
        ));
    }

    for input in cli.input.iter() {
        let name = if let Some(str) = cli.name.as_ref() {
            str.to_owned()
        } else if let Some(o) = input.file_stem() {
            o.to_str().unwrap().to_owned()
        } else {
            return Err(anyhow!("invalid input"));
        };

        let out_dir = if let Some(o) = cli.out_dir.as_ref() {
            o.to_owned()
        } else if let Some(o) = input.parent() {
            o.to_owned()
        } else {
            return Err(anyhow!("invalid input"));
        };

        if input.to_str() == Some("SOCCER") {
            let svg = generate_soccer_field();
            let path = out_dir.join(format!("{name}.svg"));
            svg::save(&path, &svg)?;
            log::info!("Soccer: exported file {path:?}");
            continue;
        }

        let atlas_params: SdfAtlasParams = Default::default();
        let chars = printable_ascii_chars();

        let font_data = std::fs::read(input).context("read font file")?;
        let font = SdfFont::create(&name, font_data, atlas_params, chars).unwrap();

        let bin_data = font.to_vec().unwrap();
        let sdfb_path = out_dir.join(format!("{name}.sdfb"));
        std::fs::write(&sdfb_path, &bin_data).context("write SDF binary file")?;
        log::info!("font {name}: exported file {sdfb_path:?}");

        let loaded = SdfFont::from_slice(&bin_data).unwrap();
        let atlas = &loaded.atlas;

        if cli.dump_atlas {
            let path = out_dir.join(format!("{name}-atlas.png"));
            atlas.export_bitmap_png(&path)?;
            log::info!("font {name}: exported atlas image {path:?}");
        }

        if cli.dump_preview {
            let path = out_dir.join(format!("{name}-preview.png"));
            atlas.export_bitmap_png(&path)?;
            println!("font {name}: exported preview image {path:?}");
        }
    }

    log::info!("done!");
    Ok(())
}

fn generate_soccer_field() -> svg::Document {
    let [width, height]: [f32; 2] = [360.0, 240.0];
    let line_width: f32 = 1.0 / 3.0;
    let [half_width, half_height] = [width * 0.5, height * 0.5];
    let mut document = svg::Document::new().set(
        "viewBox",
        (
            -half_width - 12.0,
            -half_height - 8.0,
            width + 12.0 * 2.0,
            height + 8.0 * 2.0,
        ),
    );

    let mut data = SvgData::new();

    // full court
    data = data
        .move_to((-half_width, -half_height))
        .line_by((width, 0))
        .line_by((0, height))
        .line_by((-width, 0))
        .close();

    data = data
        .move_to((-half_width + line_width, -half_height + line_width))
        .line_by((0, height - 2.0 * line_width))
        .line_by((half_width - 1.5 * line_width, 0))
        .line_to((line_width * -0.5, 30.0))
        .elliptical_arc_to((30.0, 30.0, 0.0, 1.0, 1.0, line_width * -0.5, -30.0))
        .line_to((line_width * -0.5, -half_height + line_width))
        .close();

    data = data
        .move_to((line_width * 0.5, -half_height + line_width))
        .line_to((line_width * 0.5, -30.0))
        .elliptical_arc_to((30.0, 30.0, 0.0, 0.0, 1.0, line_width * 0.5, 30.0))
        .line_to((line_width * 0.5, half_height - line_width))
        .line_by((half_width - 1.5 * line_width, 0))
        .line_by((0, -height + 2.0 * line_width))
        .close();

    data = data
        .move_to((-line_width * 0.5, -30.0 + line_width))
        .elliptical_arc_to((
            30.0 - line_width,
            30.0 - line_width,
            0.0,
            1.0,
            0.0,
            -line_width * 0.5,
            30.0 - line_width,
        ))
        .close();

    data = data
        .move_to((line_width * 0.5, 30.0 - line_width))
        .elliptical_arc_to((
            30.0 - line_width,
            30.0 - line_width,
            0.0,
            1.0,
            0.0,
            line_width * 0.5,
            -30.0 + line_width,
        ))
        .close();

    document = document.add(SvgPath::new().set("d", data));

    document
}
