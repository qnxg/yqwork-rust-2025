use anyhow::anyhow;
use salvo::{handler, macros::Extractible};

use crate::{result::RouterResult, service};

pub fn routers() -> salvo::Router {
    salvo::Router::new()
        .push(salvo::Router::with_path("login").post(login))
        .push(
            salvo::Router::with_path("auth_qrcode")
                .get(get_auth_qrcode)
                .push(
                    salvo::Router::with_path("status")
                        .push(salvo::Router::with_path("{code}").get(get_auth_qrcode_status)),
                )
                .push(
                    salvo::Router::with_path("token")
                        .push(salvo::Router::with_path("{code}").get(get_auth_qrcode_token)),
                ),
        )
}

#[handler]
async fn login(req: &mut salvo::Request) -> RouterResult {
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body")))]
    struct LoginReq {
        username: String,
        password: String,
    }
    let LoginReq { username, password } = req.extract().await?;
    let Some(user) = service::qnxg::user::get_user_by_stu_id(&username).await? else {
        return Err(anyhow!("用户名或密码错误").into());
    };
    let Some(user_password) = service::qnxg::user::get_user_password(user.id).await? else {
        return Err(anyhow!("用户名或密码错误").into());
    };
    if password != user_password {
        return Err(anyhow!("用户名或密码错误").into());
    }
    let token = service::qnxg::auth::login(user.id).await?;
    Ok(token.into())
}

#[handler]
async fn get_auth_qrcode() -> RouterResult {
    let code = service::qnxg::auth::get_auth_qrcode().await?;
    Ok(code.into())
}

#[handler]
async fn get_auth_qrcode_status(req: &mut salvo::Request) -> RouterResult {
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct AuthQrCodeStatusReq {
        code: String,
    }
    let AuthQrCodeStatusReq { code } = req.extract().await?;
    let status = service::qnxg::auth::get_auth_qrcode_status(&code).await?;
    Ok(status.into())
}

#[handler]
async fn get_auth_qrcode_token(req: &mut salvo::Request) -> RouterResult {
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct AuthQrCodeTokenReq {
        code: String,
    }
    let AuthQrCodeTokenReq { code } = req.extract().await?;
    let stu_id = service::qnxg::auth::get_auth_qrcode_info(&code).await?;
    let Some(user) = service::qnxg::user::get_user_by_stu_id(&stu_id).await? else {
        return Err(anyhow!("该用户未被添加到易千工作台").into());
    };
    let token = service::qnxg::auth::login(user.id).await?;
    Ok(token.into())
}
