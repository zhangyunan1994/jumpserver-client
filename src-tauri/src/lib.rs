// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use russh::client::{self, Handle, Msg};
use russh::keys::PublicKeyOrCertificate;
use russh::{ChannelMsg, ChannelWriteHalf, Disconnect};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex};
use url::Url;

// ==================== 数据模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Asset {
    id: String,
    title: String,
    platform_type: String,
    address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssetTreeItem {
    id: String,
    #[serde(rename = "parentId")]
    parent_id: String,
    name: String,
    #[serde(rename = "type")]
    item_type: String,
    address: String,
    #[serde(rename = "assetId")]
    asset_id: String,
}

fn json_str<'a>(item: &'a Value, keys: &[&str]) -> &'a str {
    for key in keys {
        if let Some(value) = item.get(*key).and_then(|v| v.as_str()) {
            if !value.is_empty() {
                return value;
            }
        }
    }
    ""
}

fn json_pointer_str<'a>(item: &'a Value, pointer: &str) -> Option<&'a str> {
    item.pointer(pointer).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn tree_item_type(item: &Value) -> &'static str {
    match json_pointer_str(item, "/meta/type") {
        Some("node") => "node",
        Some("asset") => "asset",
        _ => {
            if json_pointer_str(item, "/meta/data/platform_type").is_some()
                || json_pointer_str(item, "/meta/data/platform/type").is_some()
            {
                "asset"
            } else if item.get("isParent").and_then(|v| v.as_bool()).unwrap_or(false) {
                "node"
            } else {
                "asset"
            }
        }
    }
}

fn asset_platform_type(item: &Value) -> &str {
    json_pointer_str(item, "/meta/data/platform_type")
        .or_else(|| json_pointer_str(item, "/meta/data/platform/type"))
        .unwrap_or("")
}

fn asset_display_name<'a>(item: &'a Value, address: &'a str) -> &'a str {
    let name = json_pointer_str(item, "/meta/data/name")
        .or_else(|| item.get("name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()))
        .unwrap_or("");
    if !name.is_empty() {
        name
    } else if !address.is_empty() {
        address
    } else {
        "Unknown"
    }
}

fn node_display_name(item: &Value) -> String {
    if let Some(value) = json_pointer_str(item, "/meta/data/value") {
        return value.to_string();
    }
    let raw = json_str(item, &["name", "title"]);
    if let Some(pos) = raw.rfind(" (") {
        if raw.ends_with(')') {
            return raw[..pos].to_string();
        }
    }
    if raw.is_empty() {
        "未命名节点".to_string()
    } else {
        raw.to_string()
    }
}

// ==================== JumpServer API 工具 ====================

// 按字节上限截断字符串，但回退到最近的字符边界，
// 避免在多字节字符（如中文）中间切片导致 panic（release 配置 panic=abort，会直接崩溃应用）
fn truncate_utf8_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn generate_signature(secret: &str, string_to_sign: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(string_to_sign.as_bytes());
    let result = mac.finalize();
    general_purpose::STANDARD.encode(result.into_bytes())
}

fn get_auth_header(
    key_id: &str,
    secret: &str,
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
) -> String {
    let signature_headers = ["(request-target)", "accept", "date"];
    let request_target = format!("{} {}", method.to_lowercase(), path);
    let mut string_to_sign = format!("(request-target): {}\n", request_target);
    for h in &signature_headers[1..] {
        let value = headers.get(*h).cloned().unwrap_or_default();
        string_to_sign.push_str(&format!("{}: {}\n", h, value));
    }
    string_to_sign = string_to_sign.trim_end().to_string();
    let signature = generate_signature(secret, &string_to_sign);
    let headers_str = signature_headers.join(" ");
    format!(
        "Signature keyId=\"{}\",algorithm=\"hmac-sha256\",headers=\"{}\",signature=\"{}\"",
        key_id, headers_str, signature
    )
}

async fn jms_request(
    jms_url: &str,
    key_id: &str,
    secret: &str,
    method: &str,
    api_path: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let base_url = jms_url.trim_end_matches('/');
    let url = format!("{}{}", base_url, api_path);

    let parsed = Url::parse(&url).map_err(|e| format!("URL parse error: {}", e))?;
    let request_path = format!("{}{}", parsed.path(), parsed.query().map(|q| format!("?{}", q)).unwrap_or_default());

    // 与原版 Electron 保持一致的 GMT 日期格式
    let date_str = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let mut headers = HashMap::new();
    headers.insert("accept".to_string(), "application/json".to_string());
    headers.insert(
        "X-JMS-ORG".to_string(),
        "00000000-0000-0000-0000-000000000002".to_string(),
    );
    headers.insert("date".to_string(), date_str.clone());

    if body.is_some() {
        headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    let auth_header = get_auth_header(key_id, secret, method, &request_path, &headers);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        // 网络不可达时快速失败，避免 UI 无限等待
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("reqwest build error: {}", e))?;

    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap(), &url);
    for (k, v) in &headers {
        req = req.header(k, v);
    }
    req = req.header("Authorization", auth_header);

    if let Some(b) = body {
        req = req.json(&b);
    }

    let res = req.send().await.map_err(|e| {
        log::error!("[api] {} {} 请求失败: {}", method, api_path, e);
        format!("request error: {}", e)
    })?;
    let status = res.status();
    let text = res.text().await.map_err(|e| {
        log::error!("[api] {} {} 读取响应失败: {}", method, api_path, e);
        format!("read body error: {}", e)
    })?;
    // 非 2xx（如 401 密钥错误、500 服务端异常）直接给出明确错误，而非误导性的 JSON parse error
    if !status.is_success() {
        log::error!("[api] {} {} 返回 {}: {}", method, api_path, status.as_u16(), truncate_utf8_safe(&text, 500));
        return Err(format!("JumpServer API 返回 {}：{}", status.as_u16(), truncate_utf8_safe(&text, 200)));
    }
    serde_json::from_str(&text).map_err(|e| {
        log::error!("[api] {} {} JSON 解析失败: {} | {}", method, api_path, e, truncate_utf8_safe(&text, 500));
        format!("JSON parse error: {} | {}", e, truncate_utf8_safe(&text, 200))
    })
}

// ==================== SSH 连接管理 ====================

// 不校验服务器公钥的 Handler（沿用原 async-ssh2-russh::NoCheckHandler 语义，已知安全债 #1）
struct NoCheckHandler;
impl client::Handler for NoCheckHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _: &PublicKeyOrCertificate) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

struct SshConnection {
    write_tx: mpsc::UnboundedSender<Vec<u8>>,
    resize_tx: mpsc::UnboundedSender<(u32, u32)>,
    // 保存 russh 会话句柄，断开时显式发送 disconnect，避免 socket 泄漏
    handle: Handle<NoCheckHandler>,
}

#[derive(Default)]
struct AppState {
    connections: Mutex<HashMap<String, SshConnection>>,
}

async fn send_to_tab(app: &AppHandle, tab_id: &str, channel: &str, data: &str) {
    let _ = app.emit(
        channel,
        json!({ "tabId": tab_id, "data": data }),
    );
}

// 移除并显式断开指定 tab 的 SSH 会话（幂等；读写任务退出时的兜底清理也复用此函数）
async fn disconnect_tab_ssh(state: &AppState, tab_id: &str) {
    let conn = {
        let mut conns = state.connections.lock().await;
        conns.remove(tab_id)
    };
    if let Some(conn) = conn {
        // 先关闭写入端，让 writer task 退出
        drop(conn.write_tx);
        drop(conn.resize_tx);
        // 显式断开 SSH 会话，避免 channel/handle 依赖 GC 回收导致 socket 泄漏
        if let Err(e) = conn
            .handle
            .disconnect(Disconnect::ByApplication, "client closed", "en")
            .await
        {
            log::warn!("[ssh] disconnect {} failed: {}", tab_id, e);
        }
    }
}

async fn spawn_ssh_reader(
    mut stdout_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    state: Arc<AppState>,
    app: AppHandle,
    tab_id: String,
) {
    // 保存因块边界被截断的不完整 UTF-8 尾部，与下一块拼接后再解码，
    // 避免中文等多字节字符被分块切断后输出 U+FFFD 乱码。
    // recv() 返回 None = stdout 管道关闭（通道已关，EOF），退出并触发兜底清理
    let mut pending: Vec<u8> = Vec::new();
    while let Some(chunk) = stdout_rx.recv().await {
        pending.extend_from_slice(&chunk);
        match std::str::from_utf8(&pending) {
            Ok(s) => {
                let _ = send_to_tab(&app, &tab_id, "terminal-data", s).await;
                pending.clear();
            }
            Err(e) => {
                // valid_up_to 之前的字节保证为合法 UTF-8，先输出
                let valid = e.valid_up_to();
                if valid > 0 {
                    let s = std::str::from_utf8(&pending[..valid]).unwrap_or("");
                    let _ = send_to_tab(&app, &tab_id, "terminal-data", s).await;
                }
                match e.error_len() {
                    // None：尾部只是不完整（被块边界切断），保留待下一块拼接
                    None => {
                        pending.drain(..valid);
                    }
                    // Some(n)：确属非法字节（非截断），丢弃并以替换符占位（与 from_utf8_lossy 行为一致）
                    Some(bad) => {
                        let _ = send_to_tab(&app, &tab_id, "terminal-data", "\u{FFFD}").await;
                        pending.drain(..valid + bad);
                    }
                }
            }
        }
    }
    // 流结束时若仍有残余不完整字节，按 lossy 语义输出替换符，避免内容无声丢失
    if !pending.is_empty() {
        let tail = String::from_utf8_lossy(&pending).to_string();
        let _ = send_to_tab(&app, &tab_id, "terminal-data", &tail).await;
    }
    let _ = send_to_tab(&app, &tab_id, "ssh-status", "disconnected").await;
    // 读流结束（EOF/错误）意味着会话已终止：清理连接表并显式断开，防止连接残留
    disconnect_tab_ssh(&state, &tab_id).await;
}

async fn spawn_ssh_writer(
    write_half: ChannelWriteHalf<Msg>,
    state: Arc<AppState>,
    mut write_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    mut resize_rx: mpsc::UnboundedReceiver<(u32, u32)>,
    app: AppHandle,
    tab_id: String,
) {
    loop {
        tokio::select! {
            biased;
            msg = write_rx.recv() => {
                match msg {
                    Some(data) => {
                        let mut stdin = write_half.make_writer();
                        if stdin.write_all(&data).await.is_err() {
                            let _ = send_to_tab(&app, &tab_id, "ssh-status", "error").await;
                            break;
                        }
                    }
                    None => break,
                }
            }
            size = resize_rx.recv() => {
                match size {
                    Some((cols, rows)) => {
                        if write_half.window_change(cols, rows, 0, 0).await.is_err() {
                            let _ = send_to_tab(&app, &tab_id, "ssh-status", "error").await;
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    // writer 退出后连接表中的记录已无人消费，清理并断开，
    // 防止 terminal_input 继续向死 channel 无限堆积（幂等：主动关闭场景下为 no-op）
    disconnect_tab_ssh(&state, &tab_id).await;
}

async fn connect_ssh(
    state: &Arc<AppState>,
    app: AppHandle,
    tab_id: &str,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    // 先断开旧连接
    disconnect_tab_ssh(state, tab_id).await;

    let addr = format!("{}:{}", host, port);

    // 定期 keepalive：防止 NAT/防火墙静默断链或服务端超时踢掉空闲会话；
    // keepalive 失败会终止会话并触发 reader EOF 清理链路，前端状态点同步变灰
    let config = Arc::new(client::Config {
        keepalive_interval: Some(std::time::Duration::from_secs(30)),
        ..Default::default()
    });
    let mut handle = client::connect(config, addr, NoCheckHandler)
        .await
        .map_err(|e| format!("SSH connect error: {}", e))?;

    let (stdout_rx, channel_write) = match authenticate_and_open_shell(&mut handle, username, password, cols, rows).await {
        Ok(c) => c,
        Err(e) => {
            // 认证/建链失败时显式断开会话，避免 socket 泄漏
            let _ = handle
                .disconnect(Disconnect::ByApplication, "setup failed", "en")
                .await;
            return Err(e);
        }
    };

    // stdout 消息泵已在 authenticate_and_open_shell 内启动：read_half 持有通道接收端，
    // 从建链到返回之间到达的数据会先缓冲，无丢失风险
    let (write_tx, write_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (resize_tx, resize_rx) = mpsc::unbounded_channel::<(u32, u32)>();
    let conn = SshConnection {
        write_tx,
        resize_tx,
        handle,
    };

    let mut conns = state.connections.lock().await;
    conns.insert(tab_id.to_string(), conn);
    drop(conns);

    // 启动读写任务；任务退出时自行兜底清理连接表（幂等）
    tokio::spawn(spawn_ssh_reader(stdout_rx, state.clone(), app.clone(), tab_id.to_string()));
    tokio::spawn(spawn_ssh_writer(channel_write, state.clone(), write_rx, resize_rx, app.clone(), tab_id.to_string()));

    let _ = send_to_tab(&app, tab_id, "ssh-status", "connected").await;
    Ok(())
}

// 完成认证并打开 shell 通道；任一步失败由调用方负责显式断开会话。
// 返回 (stdout 字节流接收端, 通道写半端)：读半端由内部消息泵转为字节流，
// Data 与 ExtendedData 一并送入同一管道（PTY 模式下 stderr 本就并入 tty）
async fn authenticate_and_open_shell(
    handle: &mut Handle<NoCheckHandler>,
    username: &str,
    password: &str,
    cols: u32,
    rows: u32,
) -> Result<(mpsc::UnboundedReceiver<Vec<u8>>, ChannelWriteHalf<Msg>), String> {
    let auth = handle
        .authenticate_password(username, password)
        .await
        .map_err(|e| format!("SSH auth error: {}", e))?;
    if !auth.success() {
        return Err("SSH 认证失败".to_string());
    }

    let channel = handle
        .channel_open_session()
        .await
        .map_err(|e| format!("SSH channel error: {}", e))?;
    channel
        .request_pty(true, "xterm-256color", cols, rows, 0, 0, &[])
        .await
        .map_err(|e| format!("SSH pty error: {}", e))?;
    channel
        .request_shell(true)
        .await
        .map_err(|e| format!("SSH shell error: {}", e))?;

    let (mut read_half, write_half) = channel.split();
    let (tx, rx) = mpsc::unbounded_channel::<Vec<u8>>();
    // 消息泵：read_half.wait() 返回 None（通道关闭）时任务结束、tx 被 drop，
    // 下游 reader 的 recv() 随即返回 None，等价于原 ReadStream 的 EOF 语义
    tokio::spawn(async move {
        while let Some(msg) = read_half.wait().await {
            let bytes = match msg {
                ChannelMsg::Data { data } => data,
                ChannelMsg::ExtendedData { data, .. } => data,
                _ => continue,
            };
            if tx.send(bytes.to_vec()).is_err() {
                break;
            }
        }
    });

    Ok((rx, write_half))
}

// ==================== Tauri 命令 ====================

#[tauri::command]
async fn validate_credentials(
    jms_url: String,
    key_id: String,
    secret: String,
) -> Result<Value, String> {
    log::info!("[auth] 验证凭证, url={}", jms_url);
    let user = jms_request(&jms_url, &key_id, &secret, "GET", "/api/v1/users/profile/", None).await?;
    if user.get("id").is_none() {
        log::warn!("[auth] 获取用户信息失败");
        return Ok(json!({ "success": false, "error": "获取用户信息失败，请确认连接信息是否正确" }));
    }
    log::info!("[auth] 凭证验证成功, user={}", user.get("username").and_then(|v| v.as_str()).unwrap_or(""));
    Ok(json!({
        "success": true,
        "user": {
            "id": user.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "name": user.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "username": user.get("username").and_then(|v| v.as_str()).unwrap_or(""),
            "email": user.get("email").and_then(|v| v.as_str()).unwrap_or(""),
        }
    }))
}

async fn fetch_asset_tree_items(
    jms_url: &str,
    key_id: &str,
    secret: &str,
) -> Result<Vec<Value>, String> {
    let endpoints = [
        "/api/v1/perms/users/self/nodes/all-with-assets/tree/",
        "/api/v1/perms/users/self/assets/tree/",
    ];
    let mut last_err = "获取资产列表失败".to_string();
    for endpoint in endpoints {
        match jms_request(jms_url, key_id, secret, "GET", endpoint, None).await {
            Ok(data) => {
                if let Some(items) = data.as_array() {
                    if !items.is_empty() || endpoint.ends_with("assets/tree/") {
                        return Ok(items.clone());
                    }
                    last_err = "资产树为空".to_string();
                } else {
                    last_err = "获取资产列表失败".to_string();
                }
            }
            Err(err) => last_err = err,
        }
    }
    Err(last_err)
}

#[tauri::command]
async fn get_assets(jms_url: String, key_id: String, secret: String) -> Result<Value, String> {
    log::info!("[assets] 获取资产列表");
    let items = fetch_asset_tree_items(&jms_url, &key_id, &secret).await?;
    log::info!("[assets] 获取到 {} 条资产树数据", items.len());

    let mut assets: Vec<Asset> = Vec::new();
    let mut tree: Vec<AssetTreeItem> = Vec::new();
    let mut seen_assets = std::collections::HashSet::new();

    for item in &items {
        let item_type = tree_item_type(item);
        let id = json_str(item, &["id"]).to_string();
        if id.is_empty() {
            continue;
        }
        let parent_id = json_str(item, &["pId", "pid", "parentId"]).to_string();

        if item_type == "node" {
            tree.push(AssetTreeItem {
                id,
                parent_id,
                name: node_display_name(item),
                item_type: "node".to_string(),
                address: String::new(),
                asset_id: String::new(),
            });
            continue;
        }

        if asset_platform_type(item) != "linux" {
            continue;
        }

        let address = json_pointer_str(item, "/meta/data/address")
            .unwrap_or_else(|| json_str(item, &["address"]))
            .to_string();
        let name = asset_display_name(item, &address).to_string();

        tree.push(AssetTreeItem {
            id: id.clone(),
            parent_id,
            name: name.clone(),
            item_type: "asset".to_string(),
            address: address.clone(),
            asset_id: id.clone(),
        });

        if seen_assets.insert(id.clone()) {
            assets.push(Asset {
                id,
                title: name,
                platform_type: "linux".to_string(),
                address,
            });
        }
    }

    Ok(json!({ "success": true, "assets": assets, "tree": tree }))
}

#[tauri::command]
async fn connect_to_asset(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    jms_url: String,
    key_id: String,
    secret: String,
    username: String,
    asset_id: String,
    tab_id: String,
    cols: u32,
    rows: u32,
) -> Result<Value, String> {
    log::info!("[ssh] 连接资产 asset_id={}, tab_id={}", asset_id, tab_id);
    // 1. 获取资产连接账号
    let account = match jms_request(
        &jms_url,
        &key_id,
        &secret,
        "GET",
        &format!("/api/v1/perms/users/self/assets/{}/", asset_id),
        None,
    )
    .await
    {
        Ok(data) => data
            .get("permed_accounts")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or(&username)
            .to_string(),
        Err(_) => username.clone(),
    };

    // 2. 获取连接令牌
    let token = jms_request(
        &jms_url,
        &key_id,
        &secret,
        "POST",
        "/api/v1/authentication/connection-token/",
        Some(json!({
            "asset": asset_id,
            "protocol": "ssh",
            "connect_method": "ssh_guide",
            "account": account,
        })),
    )
    .await?;

    let connection_id = token
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("获取连接令牌失败")?;
    let password = token
        .get("value")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 3. 获取 Endpoint 信息（SSH 端口等），失败时 fallback 到默认值
    let endpoint = match jms_request(
        &jms_url,
        &key_id,
        &secret,
        "GET",
        "/api/v1/terminal/endpoints/smart/?protocol=ssh",
        None,
    )
    .await
    {
        Ok(ep) => {
            log::info!("[ssh] 获取 endpoint 成功: host={}, ssh_port={}",
                ep.get("host").and_then(|v| v.as_str()).unwrap_or(""),
                ep.get("ssh_port").and_then(|v| v.as_u64()).unwrap_or(2222));
            Some(ep)
        }
        Err(e) => {
            log::warn!("[ssh] 获取 endpoint 失败, 使用默认值: {}", e);
            None
        }
    };

    // 从 URL 解析主机名
    let parsed = Url::parse(jms_url.trim_end_matches('/')).map_err(|e| e.to_string())?;
    let default_host = parsed.host_str().unwrap_or("").to_string();

    // 优先使用 endpoint 中的 host，否则使用 URL 中的 host
    let ssh_host = endpoint
        .as_ref()
        .and_then(|e| e.get("host"))
        .and_then(|h| h.as_str())
        .filter(|h| !h.is_empty())
        .map(|h| h.to_string())
        .unwrap_or(default_host);

    if ssh_host.is_empty() {
        return Err("无法获取主机名".to_string());
    }

    let ssh_port = endpoint
        .as_ref()
        .and_then(|e| e.get("ssh_port"))
        .and_then(|p| p.as_u64())
        .unwrap_or(2222) as u16;

    let ssh_user = format!("JMS-{}", connection_id);

    // 4. 建立 SSH 连接
    log::info!("[ssh] 建立连接 host={}, port={}, user={}", ssh_host, ssh_port, ssh_user);
    connect_ssh(
        &state,
        app,
        &tab_id,
        &ssh_host,
        ssh_port,
        &ssh_user,
        &password,
        cols,
        rows,
    )
    .await?;

    log::info!("[ssh] 连接成功 asset_id={}, tab_id={}", asset_id, tab_id);
    Ok(json!({ "success": true, "message": format!("已连接到资产 {}", asset_id) }))
}

#[tauri::command]
async fn disconnect_ssh(state: State<'_, Arc<AppState>>, tab_id: String) -> Result<Value, String> {
    disconnect_tab_ssh(&state, &tab_id).await;
    Ok(json!({ "success": true }))
}

#[tauri::command]
async fn disconnect_all_ssh(state: State<'_, Arc<AppState>>) -> Result<Value, String> {
    disconnect_all_sessions(&state).await;
    Ok(json!({ "success": true }))
}

// 断开所有 SSH 会话并清空连接表（应用退出 / 主动断开时调用）
async fn disconnect_all_sessions(state: &AppState) {
    let conns = {
        let mut guard = state.connections.lock().await;
        std::mem::take(&mut *guard)
    };
    for (tab_id, conn) in conns {
        drop(conn.write_tx);
        drop(conn.resize_tx);
        if let Err(e) = conn
            .handle
            .disconnect(Disconnect::ByApplication, "client closed", "en")
            .await
        {
            log::warn!("[ssh] disconnect {} failed: {}", tab_id, e);
        }
    }
}

#[tauri::command]
async fn terminal_input(
    state: State<'_, Arc<AppState>>,
    tab_id: String,
    data: String,
) -> Result<(), String> {
    let conns = state.connections.lock().await;
    if let Some(conn) = conns.get(&tab_id) {
        let _ = conn.write_tx.send(data.into_bytes());
    }
    Ok(())
}

#[tauri::command]
async fn terminal_resize(
    state: State<'_, Arc<AppState>>,
    tab_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    let conns = state.connections.lock().await;
    if let Some(conn) = conns.get(&tab_id) {
        let _ = conn.resize_tx.send((cols, rows));
    }
    Ok(())
}

// ==================== 设置持久化 ====================

fn get_store_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_dir = home.join(".jumpserverclient");
    // 确保目录存在
    let _ = std::fs::create_dir_all(&config_dir);
    config_dir.join("settings.json")
}

#[tauri::command]
async fn get_settings(app: AppHandle) -> Result<Value, String> {
    let path = get_store_path();
    let store = app
        .store(&path)
        .map_err(|e| format!("open store error: {}", e))?;
    let mut settings = json!({});
    for key in ["jms_url", "key_id", "secret", "user_info", "asset_tags", "asset_order", "asset_layout", "sidebar_width", "theme", "terminal_color_scheme", "quick_commands"] {
        if let Some(value) = store.get(key) {
            settings[key] = value;
        }
    }
    Ok(settings)
}

#[tauri::command]
async fn save_settings(app: AppHandle, settings: Value) -> Result<Value, String> {
    let path = get_store_path();
    let store = app
        .store(&path)
        .map_err(|e| format!("open store error: {}", e))?;
    if let Some(obj) = settings.as_object() {
        for (k, v) in obj {
            store.set(k, v.clone());
        }
    }
    store.save().map_err(|e| format!("save store error: {}", e))?;
    Ok(json!({ "success": true }))
}

// ==================== 日志初始化 ====================

fn get_log_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_dir = home.join(".jumpserverclient");
    let _ = std::fs::create_dir_all(&config_dir);
    config_dir.join("app.log")
}

fn init_logger() {
    let log_path = get_log_path();
    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::Dispatch::new().chain(std::fs::File::create(&log_path).unwrap()))
        .apply();
}

// ==================== 应用入口 ====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logger();
    log::info!("应用启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(Arc::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            validate_credentials,
            get_assets,
            connect_to_asset,
            disconnect_ssh,
            disconnect_all_ssh,
            terminal_input,
            terminal_resize,
            get_settings,
            save_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                // 应用退出前显式断开所有 SSH 会话，
                // 让服务端立即清理会话而不是等待 TCP 超时（与 README 描述一致）
                tauri::RunEvent::Exit => {
                    let state = app_handle.state::<Arc<AppState>>();
                    tauri::async_runtime::block_on(async {
                        disconnect_all_sessions(&state).await;
                        // russh 的 disconnect() 仅将断开消息入队，实际 DISCONNECT 包由各会话事件循环异步发出；
                        // 回调返回后进程随即退出，这里留一小段时间让事件循环完成 flush，否则断开可能来不及送达
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    });
                }
                // macOS 关闭窗口后应用仍驻留：点击 Dock 图标时恢复主窗口，避免用户误以为应用已退出
                // （has_visible_windows 为 true 时由 macOS 默认行为处理，无需干预）
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { has_visible_windows: false, .. } => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        });
}
