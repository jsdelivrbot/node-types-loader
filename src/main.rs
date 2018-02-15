#![feature(conservative_impl_trait)]

#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate futures;
extern crate rayon;

use futures::{ Future, future };
use std::collections::HashMap;
use std::process::{ Command };
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use rayon::prelude::*;

type Deps = HashMap<String, String>;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PackageFile {
    dependencies: Option<Deps>,
    dev_dependencies: Option<Deps>
}

#[inline(always)]
fn load_package(name: String) -> String {
    let (command, subcommand) = ("npm", "install");
    let mut child = Command::new(command)
        .arg(subcommand)
        .arg(name.as_str())
        .arg("--save-dev")
        .spawn()
        .expect("Cannot spawn npm process");

    let ecode = child.wait();
    match ecode {
        Ok(code) => name.to_owned(),
        Err(code) => name.to_owned()
    }
}

#[inline(always)]
fn read_deps() -> Option<PackageFile> {
    let fl: Option<File> = match File::open("./package.json") {
        Ok(file) => Some(file),
        Err(err) => {
            println!("An error occured {:?}", err);
            None
        }
    };

    if fl.is_none() {
        return None;
    }

    let mut package_json = String::new();
    let mut buf_reader = BufReader::new(fl.unwrap());
    buf_reader.read_to_string(&mut package_json).expect("Cannot read from file");

    match serde_json::from_str(package_json.as_str()) {
        Ok(json) => Some(json),
        Err(err) => {
            println!("A parse error occured {:?}", err);
            None
        }
    }
}

#[inline(always)]
fn unpack_deps(deps: Option<Deps>) -> Vec<String> {
    match deps {
        Some(deps) => deps
            .keys()
            .into_iter()
            .map(|i| i.to_owned())
            .collect(),
        None => Vec::with_capacity(0),
    }
}

#[inline(always)]
fn collect_deps(deps: PackageFile) -> Vec<String> {
    let mut dependencies = unpack_deps(deps.dependencies);
    let mut dev_dependencies = unpack_deps(deps.dev_dependencies);
    let len: usize = dependencies.len() + dev_dependencies.len();
    let mut collected = Vec::with_capacity(len);
    collected.append(&mut dependencies);
    collected.append(&mut dev_dependencies);
    collected
}

#[inline(always)]
fn run(name: String) -> impl Future<Item=String, Error=String> {
    let future = future::ok::<String, String>(name);
    future.map(load_package)
}

fn main() {
    let json: PackageFile = match read_deps() {
        Some(parsed) => parsed,
        None => panic!("Cannot load types")
    };

    let packages: Vec<String> = collect_deps(json)
        .into_iter()
        .filter(|i| { !i.starts_with("@types") })
        .map(|i| { format!("@types/{}", i) })
        .collect();

    packages
        .par_iter()
        .for_each(move |p| {
            run(p.to_owned()).wait();
        });
}
