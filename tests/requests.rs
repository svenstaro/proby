mod utils;

use surf;

use utils::{Error, ProbyProcess};

/// If querying a connectable service, Proby returns 200 by default.
#[actix_rt::test]
async fn connectable_service() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(Vec::<String>::new())?;

    let mut resp_body = surf::get(&dh.selfcheck()).await?;
    assert_eq!(resp_body.status(), 200);
    assert_eq!(
        resp_body.body_string().await?,
        format!("{}:{} is connectable", dh.host, dh.port)
    );

    dh.child.kill()?;

    Ok(())
}

/// If querying an unconnectable service, Proby returns 503 by default.
#[actix_rt::test]
async fn unconnectable_service() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(Vec::<String>::new())?;

    // Generate a URL that's not connectable.
    let url = format!("{}/{}:{}", dh.url, dh.host, 1);
    let mut resp_body = surf::get(&url).await?;
    assert_eq!(resp_body.status(), 503);
    assert_eq!(
        resp_body.body_string().await?,
        format!("{}:{} is NOT connectable", dh.host, 1)
    );

    dh.child.kill()?;

    Ok(())
}

/// We can set a different good status code.
#[actix_rt::test]
async fn can_configure_good_status_code() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(Vec::<String>::new())?;

    let url = format!("{}?good=201", dh.selfcheck());
    let mut resp_body = surf::get(url).await?;
    assert_eq!(resp_body.status(), 201);
    assert_eq!(
        resp_body.body_string().await?,
        format!("{}:{} is connectable", dh.host, dh.port)
    );

    dh.child.kill()?;

    Ok(())
}

/// We can set a different bad status code.
#[actix_rt::test]
async fn can_configure_bad_status_code() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(Vec::<String>::new())?;

    let url = format!("{}?good=500", dh.selfcheck());
    let mut resp_body = surf::get(url).await?;
    assert_eq!(resp_body.status(), 500);
    assert_eq!(
        resp_body.body_string().await?,
        format!("{}:{} is connectable", dh.host, dh.port)
    );

    dh.child.kill()?;

    Ok(())
}
