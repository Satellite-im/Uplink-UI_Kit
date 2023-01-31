use futures::channel::oneshot;

use crate::{state::items::Items, warp_runner::Storage};

use warp::{
    constellation::{directory::Directory, file::File},
    error::Error,
    logging::tracing::log,
};

#[derive(Debug)]
pub enum ConstellationCmd {
    GetItemsFromCurrentDirectory {
        rsp: oneshot::Sender<Result<Items, warp::error::Error>>,
    },
    CreateNewFolder {
        folder_name: String,
        rsp: oneshot::Sender<Result<(), warp::error::Error>>,
    },
}

pub async fn handle_constellation_cmd(cmd: ConstellationCmd, storage: &mut Storage) {
    match cmd {
        ConstellationCmd::GetItemsFromCurrentDirectory { rsp } => {
            let r = initialize_items(storage);
            let _ = rsp.send(r);
        }
        ConstellationCmd::CreateNewFolder { folder_name, rsp } => {
            let r = create_new_directory(&folder_name, storage).await;
            let _ = rsp.send(r);
        }
    }
}

async fn create_new_directory(folder_name: &str, storage: &mut Storage) -> Result<(), Error> {
    let _ = storage.create_directory(folder_name, true).await?;
    log::debug!("New directory created: {:?}", folder_name);
    Ok(())
}

fn initialize_items(storage: &mut Storage) -> Result<Items, Error> {
    let current_dir = storage.current_directory()?;
    let items = current_dir.get_items();

    let mut directories: Vec<Directory> = Vec::new();
    let mut files: Vec<File> = Vec::new();

    for item in items.iter() {
        if item.is_directory() {
            let dir = item.get_directory()?;
            directories.insert(0, dir);
        } else if item.is_file() {
            let file = item.get_file()?;
            files.insert(0, file);
        };
    }

    let item = Items {
        initialized: true,
        all: items,
        directories,
        files,
    };
    Ok(item)
}
