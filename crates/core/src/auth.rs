use crate::{
    App,
    error::{Error, Result},
};
use axum::{
    extract::{FromRequestParts, Query, State},
    http::{HeaderMap, header, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openidconnect::{
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub fn random_token() -> String {
    URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}
pub fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then(|| value.to_owned())
        })
}
pub fn cookie(app: &App, name: &str, value: &str, age: i64) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={age}{}",
        if app.config.development {
            ""
        } else {
            "; Secure"
        }
    )
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub id: Uuid,
    pub admin: bool,
    pub csrf: String,
    pub session_hash: String,
}
impl FromRequestParts<App> for Actor {
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self> {
        let token =
            cookie_value(&parts.headers, "lkjmc_session").ok_or_else(Error::unauthorized)?;
        let token_hash = hash(&token);
        let row=sqlx::query("SELECT a.id,a.administrator,s.csrf FROM sessions s JOIN accounts a ON a.id=s.account_id WHERE s.token_hash=$1 AND s.expires_at>now() AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now())")
            .bind(&token_hash).fetch_optional(&app.db).await?.ok_or_else(Error::unauthorized)?;
        let csrf: String = row.get("csrf");
        if !matches!(
            parts.method,
            axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
        ) {
            use subtle::ConstantTimeEq;
            let supplied = parts
                .headers
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if !bool::from(csrf.as_bytes().ct_eq(supplied.as_bytes())) {
                return Err(Error::forbidden());
            }
            let origin = parts
                .headers
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok());
            if origin != Some(app.config.public_url.trim_end_matches('/')) {
                return Err(Error::forbidden());
            }
        }
        Ok(Self {
            id: row.get("id"),
            admin: row.get("administrator"),
            csrf,
            session_hash: token_hash,
        })
    }
}

pub async fn create_account(db: &mut PgConnection, name: &str) -> Result<Uuid> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(Error::invalid("表示名は1〜64文字です。"));
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO principals(id,kind,name) VALUES ($1,'account',$2)")
        .bind(id)
        .bind(name)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO accounts(id) VALUES ($1)")
        .bind(id)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO profiles(id,account_id) VALUES ($1,$2)")
        .bind(Uuid::new_v4())
        .bind(id)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO wallets(owner) VALUES ($1)")
        .bind(id)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO land_allowances(owner,chunks) VALUES ($1,4)")
        .bind(id)
        .execute(&mut *db)
        .await?;
    Ok(id)
}

pub async fn new_session(db: &mut PgConnection, account: Uuid) -> Result<(String, String)> {
    let token = random_token();
    let csrf = random_token();
    sqlx::query("INSERT INTO sessions(token_hash,account_id,csrf,expires_at) VALUES($1,$2,$3,now()+interval '30 days')")
        .bind(hash(&token)).bind(account).bind(&csrf).execute(db).await?;
    Ok((token, csrf))
}

type OidcClient = CoreClient<
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;
async fn oidc(app: &App) -> Result<OidcClient> {
    let issuer = app
        .config
        .oidc_issuer
        .clone()
        .ok_or_else(|| Error::unavailable("ログイン接続の設定を待っています。"))?;
    let metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(issuer).map_err(Error::internal)?,
        &app.http,
    )
    .await
    .map_err(Error::internal)?;
    Ok(CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(
            app.config
                .oidc_client_id
                .clone()
                .ok_or_else(|| Error::unavailable("OIDC client 未設定"))?,
        ),
        app.config.oidc_secret.clone().map(ClientSecret::new),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!(
            "{}/auth/callback",
            app.config.public_url.trim_end_matches('/')
        ))
        .map_err(Error::internal)?,
    ))
}
pub async fn login(State(app): State<App>) -> Result<Response> {
    let client = oidc(&app).await?;
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (url, state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("profile".into()))
        .set_pkce_challenge(challenge)
        .url();
    let browser = random_token();
    sqlx::query("INSERT INTO oidc_flows(state_hash,browser_hash,nonce,verifier,expires_at) VALUES($1,$2,$3,$4,now()+interval '10 minutes')")
        .bind(hash(state.secret())).bind(hash(&browser)).bind(nonce.secret()).bind(verifier.secret()).execute(&app.db).await?;
    Ok((
        [(
            header::SET_COOKIE,
            cookie(&app, "lkjmc_login", &browser, 600),
        )],
        Redirect::to(url.as_str()),
    )
        .into_response())
}
#[derive(Deserialize)]
pub struct Callback {
    code: Option<String>,
    state: String,
    error: Option<String>,
}
pub async fn callback(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Result<Response> {
    if query.error.is_some() {
        return Err(Error::invalid(
            "ログインがキャンセルされました。もう一度お試しください。",
        ));
    }
    let browser = cookie_value(&headers, "lkjmc_login")
        .ok_or_else(|| Error::invalid("ログイン操作が期限切れです。"))?;
    let row=sqlx::query("DELETE FROM oidc_flows WHERE state_hash=$1 AND browser_hash=$2 AND expires_at>now() RETURNING nonce,verifier")
        .bind(hash(&query.state)).bind(hash(&browser)).fetch_optional(&app.db).await?.ok_or_else(||Error::invalid("ログイン操作が期限切れ、または使用済みです。"))?;
    let client = oidc(&app).await?;
    let token = client
        .exchange_code(AuthorizationCode::new(
            query
                .code
                .ok_or_else(|| Error::invalid("認証コードがありません。"))?,
        ))
        .map_err(Error::internal)?
        .set_pkce_verifier(PkceCodeVerifier::new(row.get("verifier")))
        .request_async(&app.http)
        .await
        .map_err(Error::internal)?;
    let id_token = token
        .extra_fields()
        .id_token()
        .ok_or_else(|| Error::invalid("本人確認に必要なIDトークンがありません。"))?;
    let verifier = client.id_token_verifier();
    let nonce = Nonce::new(row.get("nonce"));
    let claims = id_token
        .claims(&verifier, &nonce)
        .map_err(Error::internal)?;
    if let Some(expected) = claims.access_token_hash() {
        let actual = AccessTokenHash::from_token(
            token.access_token(),
            id_token.signing_alg().map_err(Error::internal)?,
            id_token.signing_key(&verifier).map_err(Error::internal)?,
        )
        .map_err(Error::internal)?;
        if actual != *expected {
            return Err(Error::unauthorized());
        }
    }
    let issuer = claims.issuer().as_str();
    let subject = claims.subject().as_str();
    let name = claims
        .preferred_username()
        .map(|s| s.as_str())
        .unwrap_or("プレイヤー");
    let mut tx = app.db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("identity:{issuer}:{subject}"))
        .execute(&mut *tx)
        .await?;
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT account_id FROM identities WHERE issuer=$1 AND subject=$2")
            .bind(issuer)
            .bind(subject)
            .fetch_optional(&mut *tx)
            .await?;
    let account = match existing {
        Some(id) => id,
        None => {
            let id = create_account(&mut tx, name).await?;
            sqlx::query("INSERT INTO identities(issuer,subject,account_id,display_name) VALUES($1,$2,$3,$4)").bind(issuer).bind(subject).bind(id).bind(name).execute(&mut *tx).await?;
            id
        }
    };
    let allowed:bool=sqlx::query_scalar("SELECT merged_into IS NULL AND (banned_until IS NULL OR banned_until<now()) FROM accounts WHERE id=$1").bind(account).fetch_one(&mut *tx).await?;
    if !allowed {
        return Err(Error::forbidden());
    }
    let (session, _) = new_session(&mut tx, account).await?;
    tx.commit().await?;
    Ok((
        [
            (
                header::SET_COOKIE,
                cookie(&app, "lkjmc_session", &session, 30 * 86400),
            ),
            (header::SET_COOKIE, cookie(&app, "lkjmc_login", "", 0)),
        ],
        Redirect::to("/"),
    )
        .into_response())
}
pub async fn logout(State(app): State<App>, actor: Actor) -> Result<Response> {
    sqlx::query("DELETE FROM sessions WHERE token_hash=$1")
        .bind(actor.session_hash)
        .execute(&app.db)
        .await?;
    Ok((
        [(header::SET_COOKIE, cookie(&app, "lkjmc_session", "", 0))],
        axum::Json(serde_json::json!({"logged_out":true})),
    )
        .into_response())
}

pub async fn permission(
    db: &mut PgConnection,
    actor: Uuid,
    owner: Uuid,
    permission: &str,
) -> Result<()> {
    if actor == owner && !matches!(permission, "members" | "admin") {
        return Ok(());
    }
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM team_members m JOIN teams t ON t.id=m.team_id WHERE m.account_id=$1 AND m.team_id=$2 AND t.disbanded_at IS NULL AND (t.leader=$1 OR m.can_administer OR CASE $3 WHEN 'build' THEN m.can_build WHEN 'sell' THEN m.can_sell WHEN 'spend' THEN m.can_spend WHEN 'members' THEN m.can_manage_members ELSE false END))")
        .bind(actor).bind(owner).bind(permission).fetch_one(db).await?;
    if !allowed {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub async fn audit(
    db: &mut PgConnection,
    actor: Uuid,
    action: &str,
    resource: impl ToString,
    detail: serde_json::Value,
) -> Result<()> {
    sqlx::query("INSERT INTO audit(actor,action,resource,detail) VALUES($1,$2,$3,$4)")
        .bind(actor)
        .bind(action)
        .bind(resource.to_string())
        .bind(detail)
        .execute(db)
        .await?;
    Ok(())
}
