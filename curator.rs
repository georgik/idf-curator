extern crate json;
use clap::{Arg, App, SubCommand};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process;

use md5;

fn print_path(property_path: &std::string::String) {
    let path = Path::new(&property_path);
    let parent = path.parent().unwrap().to_str();
    print!("{}", parent.unwrap());
}

fn get_idf_id(idf_path: std::option::Option<& str>) -> String {
    let idf_path_with_slash = format!("{}", idf_path.unwrap().replace("\\","/"));
    let digest = md5::compute(idf_path_with_slash);
    return format!("esp-idf-{:x}", digest);
}

fn main() {
    let idf_tools_path_env = "IDF_TOOLS_PATH";

    let idf_tools_path = env::var(idf_tools_path_env).unwrap_or_else(|e| {
        panic!("could not find {}: {}", idf_tools_path_env, e)
    });

    let idf_json_path = idf_tools_path + "/esp_idf.json";
    let idf_slice: &str = &*idf_json_path;
    let content = fs::read_to_string(idf_slice)
    .expect("Failure");
    let mut parsed2 = json::parse(&content.to_string()).unwrap();

    let matches = App::new("My Test Program")
    .version("0.0.3")
    .author("Juraj Michalek <juraj.michalek@espressif.com>")
    .about("Maintain ESP-IDF installations")
    .subcommand(SubCommand::with_name("get")
        .arg(Arg::with_name("property")
            .short("p")
            .long("property")
            .takes_value(true)
            .help("Path to ESP-IDF installation"))
        .arg(Arg::with_name("idf-path")
            .short("i")
            .long("idf-path")
            .takes_value(true)
            .help("Path to ESP-IDF installation"))
    )
    .subcommand(SubCommand::with_name("add")
        .arg(Arg::with_name("idf-path")
                .short("i")
                .long("idf-path")
                .takes_value(true)
                .help("Path to ESP-IDF installation"))
        .arg(Arg::with_name("python")
                .short("p")
                .long("python")
                .takes_value(true)
                .help("Full path to Python interpreter binary"))
        .arg(Arg::with_name("git")
                .short("g")
                .long("git")
                .takes_value(true)
                .help("Full path to Git binary"))
        .arg(Arg::with_name("idf-version")
                .short("x")
                .long("idf-version")
                .takes_value(true)
                .help("ESP-IDF version"))

    )
    .subcommand(SubCommand::with_name("rm"))
    .subcommand(SubCommand::with_name("inspect"))
    .subcommand(SubCommand::with_name("install")
        .arg(Arg::with_name("installer")
            .short("e")
            .long("installer")
            .takes_value(true)
            .help("ESP-IDF installer tool"))
        .arg(Arg::with_name("interactive")
            .short("i")
            .long("interactive")
            .takes_value(false)
            .help("Run in interactive mode"))
        .arg(Arg::with_name("upgrade")
            .short("u")
            .long("upgrade")
            .takes_value(false)
            .help("Upgrade existing installation"))
        .arg(Arg::with_name("idf-version")
            .short("x")
            .long("idf-version")
            .takes_value(true)
            .help("ESP-IDF version"))
        .arg(Arg::with_name("idf-dir")
            .short("d")
            .long("idf-dir")
            .takes_value(true)
            .help("ESP-IDF installation directory"))
        .arg(Arg::with_name("verbose")
            .short("w")
            .long("verbose")
            .takes_value(false)
            .help("display diagnostic log after installation"))

    )
    .get_matches();


    if let Some(matches) = matches.subcommand_matches("get") {
        let property_name = matches.value_of("property").unwrap();
        let idf_path = matches.value_of("idf-path");
        if idf_path != None {
            let idf_id = get_idf_id(idf_path);
            let property_path = &parsed2["idfInstalled"][idf_id][property_name].to_string();
            print_path(property_path);
        } else {
            let property_path = &parsed2[property_name].to_string();
            print_path(property_path);
        }
    } else if let Some(_) = matches.subcommand_matches("inspect") {
        println!("{}", &content);
    } else if let Some(matches) = matches.subcommand_matches("add") {
        let python_path = matches.value_of("python").unwrap();
        let version = matches.value_of("idf-version").unwrap();
        let idf_path = matches.value_of("idf-path");
        let idf_id = get_idf_id(idf_path);
        let s_slice: &str = &*idf_id;
        let data = json::object!{
            version: version,
            python: python_path,
            path: idf_path
        };

        parsed2["idfInstalled"].insert(s_slice, data);

        fs::write(idf_slice, format!("{:#}", parsed2));
    } else if let Some(matches) = matches.subcommand_matches("install") {
        let installer = matches.value_of("installer").unwrap();
        let mut arguments : Vec<String> = [].to_vec();

        if (!matches.is_present("interactive")) {
            arguments.push("/VERYSILENT".to_string());
            arguments.push("/SUPPRESSMSGBOXES".to_string());
            arguments.push("/SP-".to_string());
            arguments.push("/NOCANCEL".to_string());
        }

        if (matches.value_of("idf-version").is_some()) {
            let version = matches.value_of("idf-version").unwrap();
            let parameter = (String::from("/IDFVERSION=") + version);
            arguments.push(parameter);
        }

        if (matches.is_present("verbose")) {
            arguments.push("/LOG=log.txt".to_string());
        }

        if (matches.value_of("idf-dir").is_some()) {
            let dir = matches.value_of("idf-dir").unwrap();
            let parameter = (String::from("/IDFDIR=") + dir);
            arguments.push(parameter);
            let path_exists = Path::new(dir).exists();

            if (matches.is_present("upgrade")) {
                if (!path_exists) {
                    println!("Unable to upgrade, path does not exist: {}", dir);
                    println!("Specify path to existing idf, or install new one without --upgrade parameter.");
                    process::exit(1);
                }
                arguments.push("/IDFUSEEXISTING=yes".to_string());
            } else {
                if (path_exists) {
                    println!("Unable to install fresh version of IDF to existing directory: {}", dir);
                    println!("Options:");
                    println!("* specify --upgrade parameter to update existing installation");
                    println!("* specify --idf-path to directory which does not exit");
                    process::exit(1);
                }
            }
        }

        let output = if cfg!(target_os = "windows") {
            println!("{} {:?}", installer, arguments);
            Command::new(installer)
                    .args(arguments)
                    .output()
                    .expect("failed to execute process")
        } else {
            Command::new("sh")
                    .arg("-c")
                    .arg("echo hello")
                    .output()
                    .expect("failed to execute process")
        };
        let data = output.stdout;
        if (matches.is_present("verbose")) {

            let output_debug = if cfg!(target_os = "windows") {
                Command::new("notepad.exe")
                        .args(&["log.txt"])
                        .output()
                        .expect("failed to execute process")
            } else {
                Command::new("sh")
                        .arg("-c")
                        .arg("echo hello")
                        .output()
                        .expect("failed to execute process")
            };
        }

    }
}
