use std::env;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use uniffi_bindgen::{
    bindings::KotlinBindingGenerator, generate_bindings, generate_external_bindings, Component,
    GenerationSettings,
};
use uniffi_bindgen_cs::{gen_cs, generate_bindings as generate_csharp_bindings};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("gen-uniffi-kotlin") => generate_uniffi_kotlin(args.collect()),
        Some("gen-uniffi-csharp") => generate_uniffi_csharp(args.collect()),
        Some(cmd) => bail!("unknown xtask command: {cmd}"),
        None => bail!("missing xtask command"),
    }
}

struct GenerateBindingsArgs {
    udl: Utf8PathBuf,
    config: Option<Utf8PathBuf>,
    out_dir: Utf8PathBuf,
}

fn parse_generate_bindings_args(
    args: Vec<String>,
    command_name: &str,
) -> Result<GenerateBindingsArgs> {
    let mut udl: Option<Utf8PathBuf> = None;
    let mut config: Option<Utf8PathBuf> = None;
    let mut out_dir: Option<Utf8PathBuf> = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--udl" => {
                let value = iter.next().context("missing value for --udl")?;
                udl = Some(Utf8PathBuf::from(value));
            }
            "--config" => {
                let value = iter.next().context("missing value for --config")?;
                config = Some(Utf8PathBuf::from(value));
            }
            "--out-dir" => {
                let value = iter.next().context("missing value for --out-dir")?;
                out_dir = Some(Utf8PathBuf::from(value));
            }
            flag => bail!("unknown flag for {command_name}: {flag}"),
        }
    }

    Ok(GenerateBindingsArgs {
        udl: udl.context("missing --udl")?,
        config,
        out_dir: out_dir.context("missing --out-dir")?,
    })
}

fn generate_uniffi_kotlin(args: Vec<String>) -> Result<()> {
    let parsed = parse_generate_bindings_args(args, "gen-uniffi-kotlin")?;

    generate_bindings(
        &parsed.udl,
        parsed.config.as_deref(),
        KotlinBindingGenerator,
        Some(&parsed.out_dir),
        None,
        None,
        false,
    )
    .context("failed to generate Kotlin UniFFI bindings")?;

    Ok(())
}

fn generate_uniffi_csharp(args: Vec<String>) -> Result<()> {
    let parsed = parse_generate_bindings_args(args, "gen-uniffi-csharp")?;

    generate_external_bindings(
        &CSharpBindingGenerator {
            try_format_code: false,
        },
        &parsed.udl,
        parsed.config.as_deref(),
        Some(&parsed.out_dir),
        None::<&Utf8Path>,
        None::<&str>,
        false,
    )
    .context("failed to generate C# UniFFI bindings")?;

    Ok(())
}

struct CSharpBindingGenerator {
    try_format_code: bool,
}

impl uniffi_bindgen::BindingGenerator for CSharpBindingGenerator {
    type Config = gen_cs::Config;

    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("csharp")) {
                Some(v) => gen_cs::Config::deserialize(v.clone())?,
                None => Default::default(),
            },
        )
    }

    fn write_bindings(
        &self,
        settings: &GenerationSettings,
        components: &[Component<Self::Config>],
    ) -> Result<()> {
        for Component { ci, config, .. } in components {
            let bindings_file = settings.out_dir.join(format!("{}.cs", ci.namespace()));
            let mut bindings = generate_csharp_bindings(config, ci)?;

            if self.try_format_code {
                if let Ok(formatted) = gen_cs::formatting::format(bindings.clone()) {
                    bindings = formatted;
                }
            }

            bindings = gen_cs::formatting::add_header(bindings);
            std::fs::write(&bindings_file, bindings)
                .with_context(|| format!("failed to write {}", bindings_file))?;
        }

        Ok(())
    }

    fn update_component_configs(
        &self,
        _settings: &GenerationSettings,
        _components: &mut Vec<Component<Self::Config>>,
    ) -> Result<()> {
        Ok(())
    }
}
