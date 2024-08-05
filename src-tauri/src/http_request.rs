use std::fs;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::render::variables_from_environment;
use crate::{render, response_err};
use base64::Engine;
use http::header::{ACCEPT, USER_AGENT};
use http::{HeaderMap, HeaderName, HeaderValue};
use log::{error, info, warn};
use mime_guess::Mime;
use reqwest::redirect::Policy;
use reqwest::Method;
use reqwest::{multipart, Url};
use tauri::{Manager, WebviewWindow};
use tokio::sync::oneshot;
use tokio::sync::watch::Receiver;
use yaak_models::models::{Cookie, CookieJar, Environment, HttpRequest, HttpResponse, HttpResponseHeader};
use yaak_models::queries::{get_workspace, update_response_if_id, upsert_cookie_jar};

pub async fn send_http_request(
    window: &WebviewWindow,
    request: HttpRequest,
    response: &HttpResponse,
    environment: Option<Environment>,
    cookie_jar: Option<CookieJar>,
    download_path: Option<PathBuf>,
    cancel_rx: &mut Receiver<bool>,
) -> Result<HttpResponse, String> {
    let environment_ref = environment.as_ref();
    let workspace = get_workspace(window, &request.workspace_id)
        .await
        .expect("Failed to get Workspace");
    let vars = variables_from_environment(&workspace, environment_ref);

    let mut url_string = render::render(&request.url, &vars);

    url_string = ensure_proto(&url_string);
    if !url_string.starts_with("http://") && !url_string.starts_with("https://") {
        url_string = format!("http://{}", url_string);
    }

    let mut client_builder = reqwest::Client::builder()
        .redirect(match workspace.setting_follow_redirects {
            true => Policy::limited(10), // TODO: Handle redirects natively
            false => Policy::none(),
        })
        .connection_verbose(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .referer(false)
        .danger_accept_invalid_certs(!workspace.setting_validate_certificates)
        .tls_info(true);

    // Add cookie store if specified
    let maybe_cookie_manager = match cookie_jar.clone() {
        Some(cj) => {
            // HACK: Can't construct Cookie without serde, so we have to do this
            let cookies = cj
                .cookies
                .iter()
                .map(|cookie| {
                    let json_cookie = serde_json::to_value(cookie).unwrap();
                    serde_json::from_value(json_cookie).expect("Failed to deserialize cookie")
                })
                .map(|c| Ok(c))
                .collect::<Vec<Result<_, ()>>>();

            let store = reqwest_cookie_store::CookieStore::from_cookies(cookies, true)
                .expect("Failed to create cookie store");
            let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(store);
            let cookie_store = Arc::new(cookie_store);
            client_builder = client_builder.cookie_provider(Arc::clone(&cookie_store));

            Some((cookie_store, cj))
        }
        None => None,
    };

    if workspace.setting_request_timeout > 0 {
        client_builder = client_builder.timeout(Duration::from_millis(
            workspace.setting_request_timeout.unsigned_abs() as u64,
        ));
    }

    let client = client_builder.build().expect("Failed to build client");

    let uri = match http::Uri::from_str(url_string.as_str()) {
        Ok(u) => u,
        Err(e) => {
            return response_err(
                response,
                format!("Failed to parse URL \"{}\": {}", url_string, e.to_string()),
                window,
            )
            .await;
        }
    };
    // Yes, we're parsing both URI and URL because they could return different errors
    let url = match Url::from_str(uri.to_string().as_str()) {
        Ok(u) => u,
        Err(e) => {
            return response_err(
                response,
                format!("Failed to parse URL \"{}\": {}", url_string, e.to_string()),
                window,
            )
            .await;
        }
    };

    let m = Method::from_bytes(request.method.to_uppercase().as_bytes())
        .expect("Failed to create method");
    let mut request_builder = client.request(m, url);

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("yaak"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

    // TODO: Set cookie header ourselves once we also handle redirects. We need to do this
    //  because reqwest doesn't give us a way to inspect the headers it sent (we have to do
    //  everything manually to know that).
    // if let Some(cookie_store) = maybe_cookie_store.clone() {
    //     let values1 = cookie_store.get_request_values(&url);
    //     let raw_value = cookie_store.get_request_values(&url)
    //         .map(|(name, value)| format!("{}={}", name, value))
    //         .collect::<Vec<_>>()
    //         .join("; ");
    //     headers.insert(
    //         COOKIE,
    //         HeaderValue::from_str(&raw_value).expect("Failed to create cookie header"),
    //     );
    // }

    for h in request.headers {
        if h.name.is_empty() && h.value.is_empty() {
            continue;
        }

        if !h.enabled {
            continue;
        }

        let name = render::render(&h.name, &vars);
        let value = render::render(&h.value, &vars);

        let header_name = match HeaderName::from_bytes(name.as_bytes()) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to create header name: {}", e);
                continue;
            }
        };
        let header_value = match HeaderValue::from_str(value.as_str()) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to create header value: {}", e);
                continue;
            }
        };

        headers.insert(header_name, header_value);
    }

    if let Some(b) = &request.authentication_type {
        let empty_value = &serde_json::to_value("").unwrap();
        let a = request.authentication;

        if b == "basic" {
            let raw_username = a
                .get("username")
                .unwrap_or(empty_value)
                .as_str()
                .unwrap_or("");
            let raw_password = a
                .get("password")
                .unwrap_or(empty_value)
                .as_str()
                .unwrap_or("");
            let username = render::render(raw_username, &vars);
            let password = render::render(raw_password, &vars);

            let auth = format!("{username}:{password}");
            let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(auth);
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Basic {}", encoded)).unwrap(),
            );
        } else if b == "bearer" {
            let raw_token = a.get("token").unwrap_or(empty_value).as_str().unwrap_or("");
            let token = render::render(raw_token, &vars);
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            );
        }
    }

    let mut query_params = Vec::new();
    for p in request.url_parameters {
        if !p.enabled || p.name.is_empty() {
            continue;
        }
        query_params.push((
            render::render(&p.name, &vars),
            render::render(&p.value, &vars),
        ));
    }
    request_builder = request_builder.query(&query_params);

    if let Some(body_type) = &request.body_type {
        let empty_string = &serde_json::to_value("").unwrap();
        let empty_bool = &serde_json::to_value(false).unwrap();
        let request_body = request.body;

        if request_body.contains_key("text") {
            let raw_text = request_body
                .get("text")
                .unwrap_or(empty_string)
                .as_str()
                .unwrap_or("");
            let body = render::render(raw_text, &vars);
            request_builder = request_builder.body(body);
        } else if body_type == "application/x-www-form-urlencoded"
            && request_body.contains_key("form")
        {
            let mut form_params = Vec::new();
            let form = request_body.get("form");
            if let Some(f) = form {
                for p in f.as_array().unwrap_or(&Vec::new()) {
                    let enabled = p
                        .get("enabled")
                        .unwrap_or(empty_bool)
                        .as_bool()
                        .unwrap_or(false);
                    let name = p
                        .get("name")
                        .unwrap_or(empty_string)
                        .as_str()
                        .unwrap_or_default();
                    if !enabled || name.is_empty() {
                        continue;
                    }
                    let value = p
                        .get("value")
                        .unwrap_or(empty_string)
                        .as_str()
                        .unwrap_or_default();
                    form_params.push((render::render(name, &vars), render::render(value, &vars)));
                }
            }
            request_builder = request_builder.form(&form_params);
        } else if body_type == "binary" && request_body.contains_key("filePath") {
            let file_path = request_body
                .get("filePath")
                .ok_or("filePath not set")?
                .as_str()
                .unwrap_or_default();

            match fs::read(file_path).map_err(|e| e.to_string()) {
                Ok(f) => {
                    request_builder = request_builder.body(f);
                }
                Err(e) => {
                    return response_err(response, e, window).await;
                }
            }
        } else if body_type == "multipart/form-data" && request_body.contains_key("form") {
            let mut multipart_form = multipart::Form::new();
            if let Some(form_definition) = request_body.get("form") {
                for p in form_definition.as_array().unwrap_or(&Vec::new()) {
                    let enabled = p
                        .get("enabled")
                        .unwrap_or(empty_bool)
                        .as_bool()
                        .unwrap_or(false);
                    let name_raw = p
                        .get("name")
                        .unwrap_or(empty_string)
                        .as_str()
                        .unwrap_or_default();

                    if !enabled || name_raw.is_empty() {
                        continue;
                    }

                    let file_path = p
                        .get("file")
                        .unwrap_or(empty_string)
                        .as_str()
                        .unwrap_or_default();
                    let value_raw = p
                        .get("value")
                        .unwrap_or(empty_string)
                        .as_str()
                        .unwrap_or_default();

                    let name = render::render(name_raw, &vars);
                    let mut part = if file_path.is_empty() {
                        multipart::Part::text(render::render(value_raw, &vars))
                    } else {
                        match fs::read(file_path) {
                            Ok(f) => multipart::Part::bytes(f),
                            Err(e) => {
                                return response_err(response, e.to_string(), window).await;
                            }
                        }
                    };

                    let ct_raw = p
                        .get("contentType")
                        .unwrap_or(empty_string)
                        .as_str()
                        .unwrap_or_default();

                    // Set or guess mimetype
                    if !ct_raw.is_empty() {
                        let content_type = render::render(ct_raw, &vars);
                        part = part
                            .mime_str(content_type.as_str())
                            .map_err(|e| e.to_string())?;
                    } else if !file_path.is_empty() {
                        let default_mime = Mime::from_str("application/octet-stream").unwrap();
                        let mime = mime_guess::from_path(file_path).first_or(default_mime);
                        part = part
                            .mime_str(mime.essence_str())
                            .map_err(|e| e.to_string())?;
                    }

                    // Set fil path if not empty
                    if !file_path.is_empty() {
                        let filename = PathBuf::from(file_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_string();
                        part = part.file_name(filename);
                    }

                    multipart_form = multipart_form.part(name, part);
                }
            }
            headers.remove("Content-Type"); // reqwest will add this automatically
            request_builder = request_builder.multipart(multipart_form);
        } else {
            warn!("Unsupported body type: {}", body_type);
        }
    }

    // Add headers last, because previous steps may modify them
    request_builder = request_builder.headers(headers);

    let sendable_req = match request_builder.build() {
        Ok(r) => r,
        Err(e) => {
            return response_err(response, e.to_string(), window).await;
        }
    };

    let start = std::time::Instant::now();

    let (resp_tx, resp_rx) = oneshot::channel();

    tokio::spawn(async move {
        let _ = resp_tx.send(client.execute(sendable_req).await);
    });

    let raw_response = tokio::select! {
        Ok(r) = resp_rx => {r}
        _ = cancel_rx.changed() => {
            return response_err(response, "Request was cancelled".to_string(), window).await;
        }
    };

    match raw_response {
        Ok(v) => {
            let mut response = response.clone();
            response.elapsed_headers = start.elapsed().as_millis() as i32;
            let response_headers = v.headers().clone();
            response.status = v.status().as_u16() as i32;
            response.status_reason = v.status().canonical_reason().map(|s| s.to_string());
            response.headers = response_headers
                .iter()
                .map(|(k, v)| HttpResponseHeader {
                    name: k.as_str().to_string(),
                    value: v.to_str().unwrap_or_default().to_string(),
                })
                .collect();
            response.url = v.url().to_string();
            response.remote_addr = v.remote_addr().map(|a| a.to_string());
            response.version = match v.version() {
                reqwest::Version::HTTP_09 => Some("HTTP/0.9".to_string()),
                reqwest::Version::HTTP_10 => Some("HTTP/1.0".to_string()),
                reqwest::Version::HTTP_11 => Some("HTTP/1.1".to_string()),
                reqwest::Version::HTTP_2 => Some("HTTP/2".to_string()),
                reqwest::Version::HTTP_3 => Some("HTTP/3".to_string()),
                _ => None,
            };

            let content_length = v.content_length();
            let body_bytes = v.bytes().await.expect("Failed to get body").to_vec();
            response.elapsed = start.elapsed().as_millis() as i32;

            // Use content length if available, otherwise use body length
            response.content_length = match content_length {
                Some(l) => Some(l as i32),
                None => Some(body_bytes.len() as i32),
            };

            {
                // Write body to FS
                let dir = window.app_handle().path().app_data_dir().unwrap();
                let base_dir = dir.join("responses");
                create_dir_all(base_dir.clone()).expect("Failed to create responses dir");
                let body_path = match response.id.is_empty() {
                    false => base_dir.join(response.id.clone()),
                    true => base_dir.join(uuid::Uuid::new_v4().to_string()),
                };
                let mut f = File::options()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(&body_path)
                    .expect("Failed to open file");
                f.write_all(body_bytes.as_slice())
                    .expect("Failed to write to file");
                response.body_path = Some(
                    body_path
                        .to_str()
                        .expect("Failed to get body path")
                        .to_string(),
                );
            }

            response = update_response_if_id(window, &response)
                .await
                .expect("Failed to update response");

            // Copy response to the download path, if specified
            match (download_path, response.body_path.clone()) {
                (Some(dl_path), Some(body_path)) => {
                    info!("Downloading response body to {}", dl_path.display());
                    fs::copy(body_path, dl_path)
                        .expect("Failed to copy file for response download");
                }
                _ => {}
            };

            // Add cookie store if specified
            if let Some((cookie_store, mut cookie_jar)) = maybe_cookie_manager {
                // let cookies = response_headers.get_all(SET_COOKIE).iter().map(|h| {
                //     println!("RESPONSE COOKIE: {}", h.to_str().unwrap());
                //     cookie_store::RawCookie::from_str(h.to_str().unwrap())
                //         .expect("Failed to parse cookie")
                // });
                // store.store_response_cookies(cookies, &url);

                let json_cookies: Vec<Cookie> = cookie_store
                    .lock()
                    .unwrap()
                    .iter_any()
                    .map(|c| {
                        let json_cookie = serde_json::to_value(&c).expect("Failed to serialize cookie");
                        serde_json::from_value(json_cookie).expect("Failed to deserialize cookie")
                    })
                    .collect::<Vec<_>>();
                cookie_jar.cookies = json_cookies;
                if let Err(e) = upsert_cookie_jar(window, &cookie_jar).await {
                    error!("Failed to update cookie jar: {}", e);
                };
            }

            Ok(response)
        }
        Err(e) => response_err(response, e.to_string(), window).await,
    }
}

fn ensure_proto(url_str: &str) -> String {
    if url_str.starts_with("http://") || url_str.starts_with("https://") {
        return url_str.to_string();
    }

    // Url::from_str will fail without a proto, so add one
    let parseable_url = format!("http://{}", url_str);
    if let Ok(u) = Url::from_str(parseable_url.as_str()) {
        match u.host() {
            Some(host) => {
                let h = host.to_string();
                // These TLDs force HTTPS
                if h.ends_with(".app") || h.ends_with(".dev") || h.ends_with(".page") {
                    return format!("https://{url_str}");
                }
            }
            None => {}
        }
    }

    format!("http://{url_str}")
}
