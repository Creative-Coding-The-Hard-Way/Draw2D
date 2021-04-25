use std::env;
use std::fs;
use std::path::Path;

const PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");

fn main() {
    let doc_folder = Path::new("target").join("doc");

    fs::create_dir_all(&doc_folder).expect("unable to create the doc folder!");

    let path = doc_folder.join("index.html");
    fs::write(
        &path,
        format!(
            "
            <!doctype html>
            <html>
                <head>
                    <meta
                        http-equiv=\"refresh\"
                        content=\"0; URL=./{}/index.html\"/>
                </head>
                <body>
                </body>
            </html>
            ",
            PACKAGE_NAME,
        ),
    )
    .expect("unable to generate documentation index file!");
}
