use bollard::{
    Docker,
    query_parameters::{ListContainersOptionsBuilder, ListImagesOptionsBuilder},
};
use std::error::Error;
use tokio::{fs, process::Command};
// use futures_util::future::FutureExt; // Not strictly needed for this snippet

async fn check_version(docker: &Docker) {
    let version = docker.version().await.unwrap();
    let containers = docker
        .list_containers(Some(ListContainersOptionsBuilder::default().build()))
        .await
        .unwrap();
    let images = docker
        .list_images(Some(ListImagesOptionsBuilder::default().build()))
        .await
        .unwrap();

    fs::create_dir_all("sboms").await.unwrap();
    for container in containers.iter() {
        println!("{:#?}", container.image_id);
        match container.image_id {
            Some(ref v) => {
                if let Some(hash) = v.strip_prefix("sha256:") {
                    match fs::try_exists(format!("sboms/{}.json", hash)).await {
                        Ok(false) => run_syft(hash).await.expect("Error"),
                        Ok(true) => (),
                        Err(e) => eprintln!("Failed to check if file exists : {}", e),
                    }
                }
            }
            None => (),
        }
    }
    println!("{:#?}", containers);
    println!("{:#?}", version);
    println!("{:#?}", images);
}

async fn run_syft(image_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("syft")
        .arg(image_id)
        .arg("-o")
        .arg("json")
        .arg("--file")
        .arg(format!("sboms/{}.json", image_id))
        .output()
        .await?;
    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout)?;
        println!("Syft scan successful!");
        println!("{}", stdout);
    } else {
        let stderr = str::from_utf8(&output.stderr)?;
        eprintln!("Syft failed with status {}", output.status);
        eprintln!("Error details: {}", stderr);
    }
    Ok(())
}

async fn run_grype() -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir("sboms").await?;
    while let Some(sbom) = entries.next_entry().await? {
        println!("{:#?}", sbom.path());
        let output = Command::new("grype")
            .arg("--output")
            .arg("json")
            .arg(sbom.path())
            .output()
            .await?;
        if output.status.success() {
            let stdout = str::from_utf8(&output.stdout)?;
            println!("Syft scan successful!");
            println!("{}", stdout);
        } else {
            let stderr = str::from_utf8(&output.stderr)?;
            eprintln!("Syft failed with status {}", output.status);
            eprintln!("Error details: {}", stderr);
        }
    }
    Ok(())
    //  let output = Command::new("grype")
    //      .arg(arg)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let docker = Docker::connect_with_local_defaults()?;
    check_version(&docker).await;
    run_grype().await?;
    Ok(())
}
