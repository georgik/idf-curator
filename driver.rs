use clap::Arg;
use clap_nested::{Command, Commander, MultiCommand};
use std::path::Path;
use std::io::Cursor;
use wmi::*;
use std::collections::HashMap;
use wmi::Variant;
use tokio::runtime::Runtime;
use tokio::task;
use tokio::runtime::Handle;



type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


fn get_driver_property(property_name: String, query: String) -> Result<()>  {
    let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2", COMLibrary::new()?.into())?;
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query).unwrap();
    for driver_item in results {
        let property_value = &driver_item[&property_name];

        if let Variant::String(value) = property_value {
            print!("{}", value )
        }

    }
    Ok(())
}

fn get_installed_driver_property(property_name: String) -> Result<()> {
    // Driver classes: https://docs.microsoft.com/en-us/windows-hardware/drivers/install/system-defined-device-setup-classes-available-to-vendors?redirectedfrom=MSDN
    return get_driver_property(property_name, "SELECT * FROM Win32_PnPEntity WHERE ClassGuid=\"{4d36e978-e325-11ce-bfc1-08002be10318}\"".to_string());
}

fn get_missing_driver_property(property_name: String) -> Result<()> {
    // https://stackoverflow.com/questions/11367639/get-a-list-of-devices-with-missing-drivers-using-powershell
    return get_driver_property(property_name, "SELECT * FROM Win32_PnPEntity WHERE ConfigManagerErrorCode>0".to_string());
}

async fn fetch_url(url: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create("driver.zip")?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn download_zip(url: String, output: String) -> Result<()> {
    if Path::new(&output).exists() {
        println!("Using cached driver.");
        return Ok(());
    }
    fetch_url(url).await
}

fn download_driver() -> Result<()> {
    let driver_archive = "driver.zip".to_string();
    let handle = Handle::current().clone();
    let th = std::thread::spawn(move || {
        handle.block_on(download_zip("https://www.silabs.com/documents/public/software/CP210x_Universal_Windows_Driver.zip".to_string(), driver_archive)).unwrap();
    });
    th.join().unwrap();
    Ok(())
}

pub fn get_cmd<'a>() -> Command<'a, str> {
    Command::new("get")
        .description("Get information about drivers")
        .options(|app| {
            app.arg(
                Arg::with_name("property")
                    .short("p")
                    .long("property")
                    .help("Filter result for property name")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("missing")
                    .short("m")
                    .long("missing")
                    .help("Display missing drivers")
            )
        })
        .runner(|_args, matches| {
            let property_name =  matches.value_of("property").unwrap().to_string();
            if matches.is_present("missing") {
                get_missing_driver_property(property_name).unwrap();
            } else {
                get_installed_driver_property(property_name).unwrap();
            }
            Ok(())
        })
}

fn  get_runner(_args:&str, matches:&clap::ArgMatches<'_>)  -> std::result::Result<(), clap::Error> {
    let mut arguments : Vec<String> = [].to_vec();
    download_driver().unwrap();
    Ok(())
    // if !matches.is_present("installer")  {
    //     download_installer();
    // }

    // if !matches.is_present("interactive") {
    //     arguments.push("/VERYSILENT".to_string());
    //     arguments.push("/SUPPRESSMSGBOXES".to_string());
    //     arguments.push("/SP-".to_string());
    //     arguments.push("/NOCANCEL".to_string());
    // }

    // if matches.is_present("idf-version") {
    //     let version = matches.value_of("idf-version").unwrap();
    //     let parameter = String::from("/IDFVERSION=") + version;
    //     arguments.push(parameter);
    // }

    // if matches.is_present("verbose") {
    //     arguments.push("/LOG=log.txt".to_string());
    // }


    // let output = if cfg!(target_os = "windows") {
    //     println!("{} {:?}", get_installer(matches), arguments);
    //     std::process::Command::new(get_installer(matches))
    //             .args(arguments)
    //             .output()
    //             .expect("failed to execute process")
    // } else {
    //     std::process::Command::new("sh")
    //             .arg("-c")
    //             .arg("echo hello")
    //             .output()
    //             .expect("failed to execute process")
    // };
    // let _data = output.stdout;
    // if matches.is_present("verbose") {

    //     if cfg!(target_os = "windows") {
    //         std::process::Command::new("notepad.exe")
    //                 .args(&["log.txt"])
    //                 .output()
    //                 .expect("failed to execute process")
    //     } else {
    //         std::process::Command::new("sh")
    //                 .arg("-c")
    //                 .arg("echo hello")
    //                 .output()
    //                 .expect("failed to execute process")
    //     };
    // }

}

pub fn get_install_cmd<'a>() -> Command<'a, str> {
    Command::new("install")
        .description("Install driver")
        .options(|app| {
            app.arg(
                Arg::with_name("installer")
                    .short("e")
                    .long("installer")
                    .help("Path to installer binary"),
            )
            .arg(
                Arg::with_name("interactive")
                    .short("i")
                    .long("interactive")
                    .help("Run installation in interactive mode"),

            )
            .arg(
                Arg::with_name("verbose")
                    .short("w")
                    .long("verbose")
                    .takes_value(false)
                    .help("display diagnostic log after installation"))
        })
        .runner(|_args,matches|get_runner(_args, matches)
        )
}


pub fn get_multi_cmd<'a>() -> MultiCommand<'a, str, str> {
    let multi_cmd: MultiCommand<str, str> = Commander::new()
    .add_cmd(get_cmd())
    .add_cmd(get_install_cmd())
    .into_cmd("driver")

    // Optionally specify a description
    .description("Detection of Antivirus and handling exception registration.");

    return multi_cmd;
}
