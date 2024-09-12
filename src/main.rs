use std::{collections::HashMap, env, fs::File, io::Write, path::PathBuf, process::Command};

pub type BoxedError = Box<dyn std::error::Error>;

pub const SHOULD_RUN: bool = true;
pub const DEBUG: bool = false;

fn main() -> Result<(), BoxedError> {
    let conf = get_conf();

    println!("making workspace for {}", &conf.project_base);

    let map = mappings(&conf.project_base);

    make(&conf, map)?;

    println!("done!");

    Ok(())
}

pub struct Conf {
    location: PathBuf,
    project_base: String,
}

fn get_conf() -> Conf {
    let options: Vec<String> = env::args().collect();

    Conf {
        location: options[1].clone().into(),
        project_base: options[2].clone(),
    }
}

pub enum Dep {
    Remote((&'static str, Option<Vec<&'static str>>)),
    Local(String),
}

impl Dep {
    fn name(&self) -> String {
        match self {
            Dep::Remote((name, _)) => name.to_string(),
            Dep::Local(name) => name.clone(),
        }
    }

    fn add_flag(&self) -> Option<&'static str> {
        match self {
            Dep::Remote(_) => None,
            Dep::Local(_) => Some("--path"),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Crate {
    Bin(&'static str),
    Lib(&'static str),
}

impl Crate {
    fn name(&self) -> &'static str {
        match self {
            Crate::Bin(name) => name,
            Crate::Lib(name) => name,
        }
    }

    fn crate_flag(&self) -> &'static str {
        match self {
            Crate::Bin(_) => "--bin",
            Crate::Lib(_) => "--lib",
        }
    }
}

fn make(conf: &Conf, map: HashMap<Crate, Vec<Dep>>) -> Result<(), BoxedError> {
    let dst = conf.location.join(&conf.project_base);
    let loc_dir = conf
        .location
        .to_str()
        .ok_or("unable to make workspace root")?;

    let dir = dst.to_str().ok_or("unable to make project base")?;

    handle_command(Command::new("mkdir").args(["-p", dir]))?;

    if let Ok(mut file) = File::create_new(format!("{}/Cargo.toml", loc_dir)) {
        file.write_all(include_bytes!("../Workspace.toml"))?;
    };

    println!("generating crates");
    for (crate_, _) in map.iter() {
        let crate_fullname = format!("{}-{}", &conf.project_base, crate_.name());

        handle_command(Command::new("cargo").current_dir(dir).args([
            "new",
            crate_.crate_flag(),
            &crate_fullname,
        ]))?;
    }

    println!("adding dependencies");
    for (crate_, deps) in map.iter() {
        let crate_fullname = format!("{}-{}", &conf.project_base, crate_.name());
        let crate_full_dir = dst.join(&crate_fullname);

        for dep in deps.iter() {
            let mut args: Vec<String> = vec!["add".to_string()];

            if let Some(flag) = dep.add_flag() {
                args.push(flag.to_string());
            }

            args.push(dep.name().to_string());

            if let Dep::Remote((_, Some(features))) = &dep {
                args.push("--features".to_string());

                args.push(features.join(","));
            }

            handle_command(
                Command::new("cargo")
                    .current_dir(&crate_full_dir)
                    .args(args),
            )?;
        }
    }

    Ok(())
}

fn mappings(project_base: &str) -> HashMap<Crate, Vec<Dep>> {
    let mut map = HashMap::new();

    map.insert(
        Crate::Lib("rs"),
        vec![
            Dep::Local(format!("../{project_base}-http")),
            Dep::Remote(("reqwest", Some(vec!["json"]))),
            Dep::Remote(("tracing", None)),
            Dep::Remote(("url", Some(vec!["serde"]))),
        ],
    );

    map.insert(
        Crate::Lib("http"),
        vec![
            Dep::Local(format!("../{project_base}-kernel")),
            Dep::Remote(("chrono", Some(vec!["serde"]))),
            Dep::Remote(("serde", Some(vec!["derive"]))),
            Dep::Remote(("serde_json", Some(vec!["derive"]))),
            Dep::Remote(("uuid", Some(vec!["serde", "v4"]))),
        ],
    );

    map.insert(
        Crate::Bin("api"),
        vec![
            Dep::Local(format!("../{project_base}-http")),
            Dep::Local(format!("../{project_base}-kernel")),
            Dep::Local(format!("../{project_base}-postgres")),
            Dep::Local(format!("../{project_base}-repository")),
            Dep::Remote(("async-trait", None)),
            Dep::Remote((
                "axum",
                Some(vec![
                    "http1",
                    "json",
                    "macros",
                    "matched-path",
                    "original-uri",
                    "tower-log",
                    "query",
                ]),
            )),
            Dep::Remote(("axum-macros", None)),
            Dep::Remote(("axum-test", None)),
            Dep::Remote(("tokio", Some(vec!["full"]))),
            Dep::Remote(("tower-http", Some(vec!["cors"]))),
            Dep::Remote(("tracing", None)),
            Dep::Remote(("uuid", Some(vec!["serde", "v4"]))),
        ],
    );

    map.insert(
        Crate::Lib("kernel"),
        vec![
            Dep::Remote(("async-trait", None)),
            Dep::Remote(("chrono", Some(vec!["serde"]))),
            Dep::Remote(("tracing", None)),
            Dep::Remote(("uuid", Some(vec!["serde", "v4"]))),
        ],
    );

    map.insert(
        Crate::Lib("services"),
        vec![
            Dep::Local(format!("../{project_base}-kernel")),
            Dep::Local(format!("../{project_base}-repository")),
            Dep::Remote(("async-trait", None)),
            Dep::Remote(("tokio", Some(vec!["full"]))),
            Dep::Remote(("chrono", Some(vec!["serde"]))),
            Dep::Remote(("tracing", None)),
            Dep::Remote(("uuid", Some(vec!["serde", "v4"]))),
        ],
    );

    map.insert(
        Crate::Lib("repository"),
        vec![
            Dep::Local(format!("../{project_base}-kernel")),
            Dep::Remote(("async-trait", None)),
            Dep::Remote(("chrono", Some(vec!["serde"]))),
            Dep::Remote(("tracing", None)),
            Dep::Remote(("uuid", Some(vec!["serde", "v4"]))),
        ],
    );

    map.insert(
        Crate::Lib("postgres"),
        vec![
            Dep::Local(format!("../{project_base}-repository")),
            Dep::Remote(("async-trait", None)),
            Dep::Remote(("chrono", Some(vec!["serde"]))),
            Dep::Remote((
                "sqlx",
                Some(vec![
                    "chrono",
                    "migrate",
                    "postgres",
                    "runtime-tokio-rustls",
                    "uuid",
                ]),
            )),
            Dep::Remote(("tokio", Some(vec!["full"]))),
            Dep::Remote(("tracing", None)),
            Dep::Remote(("uuid", Some(vec!["serde", "v4"]))),
        ],
    );

    map
}

fn handle_command(command: &mut Command) -> Result<(), std::io::Error> {
    if DEBUG {
        println!("running:\n\t{:?}", &command);
    }

    if SHOULD_RUN {
        let output = command.output()?;

        if DEBUG {
            println!("result:\n\t{:?}", output);
        }
    }

    Ok(())
}
