use futures::future;
use std::future::Future;

pub async fn handle_promises<T>(promises: Vec<impl Future<Output = Result<T, String>>>) -> Vec<T> {
    future::join_all(promises)
        .await
        .into_iter()
        .filter_map(|res| match res {
            Ok(data) => Some(data),
            Err(msg) => {
                eprintln!("{msg}");
                None
            },
        })
        .collect()
}
