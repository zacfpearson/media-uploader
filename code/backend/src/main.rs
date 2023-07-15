#![deny(warnings)]
use pretty_env_logger;

use bytes::{BufMut};
use futures_util::{StreamExt, TryStreamExt};
use std::env;
use warp::{multipart::FormData, Filter, Rejection, Reply};

use mongodb::{bson::doc, gridfs::{GridFsBucket, FilesCollectionDocument}, Client};

async fn uploadupload_handler(mut fd: FormData, bucket: GridFsBucket) -> std::result::Result<impl Reply, Rejection> {
    while let Some(possible_part) = fd.next().await {
        match possible_part {
            Ok(part) => {
                if let Some(filename) = part.filename() {
                    let filename_clone = filename.to_owned(); //needed to remove the borrow from part
                    println!("got file: {}", filename);

                    let value = part
                        .stream()
                        .try_fold(Vec::new(), |mut vec, data| {
                            vec.put(data);
                            async move { Ok(vec) }
                        })
                        .await
                        .map_err(|e| {
                            eprintln!("reading file error: {}", e);
                            warp::reject::reject()
                        })?;
                        
                    bucket
                        .upload_from_futures_0_3_reader(filename_clone, &value[..], None)
                        .await
                        .expect("should be able to download data to bucket");
                } 
            },
            Err(_) => {
                warp::reject::reject(); //todo: different error
            }
        }
    }
    Ok("saved data")
}

async fn delete_handler(filename: String, bucket: GridFsBucket) -> std::result::Result<impl Reply, Rejection> {
    println!("got file: {}", filename);
    let filter = doc!{"filename": filename.clone()};
    println!("finding file");
    match bucket.find(filter, None).await {
        Ok(cursor) => {
            println!("found file");
            let file_collections: Result<Vec<FilesCollectionDocument>,_> = cursor.try_collect().await;
            match file_collections {
                Ok(metas) => {
                    let id = metas[0].id.clone();
        
                bucket
                    .delete(id)
                    .await
                    .expect("should be able to delete data from bucket");
                },
                Err(_) => {
                    warp::reject::reject(); //todo: different error
                }
            }
        },
        Err(_) => {
            warp::reject::reject(); //todo: different error
        }
    }
    Ok("deleted data")
}

#[tokio::main]    
async fn main() {
    pretty_env_logger::init();

    let mongo_endpoint = env::var("ENDPOINT").expect("$ENDPOINT is not set");
    let mongo_table = env::var("TABLE").expect("$TABLE is not set");

    let client = Client::with_uri_str(&mongo_endpoint).await.expect("should be able to setup MondoDB client");
    let db = client.database(&mongo_table);
    let bucket = db.gridfs_bucket(None);

    let bucket = warp::any().map(move || bucket.clone());

    let media = warp::path("uploader");
    let delete_media = warp::path("deleter");
    let media_routes = media
        .and(warp::post())
        .and(warp::multipart::form().max_length(300_000_000))
        .and(bucket.clone())
        .and_then(uploadupload_handler)
        .or(
            media.and(warp::options().map(warp::reply))
        )
        .or(
            delete_media
            .and(warp::post())
            .and(warp::path::param())
            .and(bucket.clone())
            .and_then(delete_handler)
        )
        .or(
            delete_media.and(warp::options().map(warp::reply))
        );

    let web = warp::path("web").and(warp::fs::dir("web"));
        
    let routes = web.or(
        warp::path("api").and(
            warp::path("media").and(
                warp::path("v1").and(
                    media_routes
                )
            )
        )
    );

    warp::serve(routes).run(([127, 0, 0, 1], 80)).await;

}
