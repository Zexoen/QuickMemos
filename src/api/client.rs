use crate::api::error::ApiError;
use crate::api::models::*;
use reqwest::Client;

pub struct MemosClient {
    client: Client,
    base_url: String,
    token: String,
}

impl MemosClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");

        let base_url = base_url.trim_end_matches('/').to_string();

        Self {
            client,
            base_url,
            token: token.to_string(),
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    #[allow(dead_code)]
    pub async fn test_connection(&self) -> Result<InstanceProfile, ApiError> {
        let resp = self
            .client
            .get(self.api_url("/instance/profile"))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            401 => Err(ApiError::Unauthorized),
            404 => Err(ApiError::NotFound),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ApiError::Server(format!("{code}: {body}")))
            }
        }
    }

    pub async fn list_memos(
        &self,
        page_size: u32,
        page_token: &str,
        filter: Option<&str>,
        state: &str,
    ) -> Result<ListMemosResponse, ApiError> {
        let mut params: Vec<(&str, String)> = vec![
            ("pageSize", page_size.to_string()),
            ("state", state.to_string()),
        ];
        if !page_token.is_empty() {
            params.push(("pageToken", page_token.to_string()));
        }
        if let Some(f) = filter {
            if !f.is_empty() {
                params.push(("filter", f.to_string()));
            }
        }

        let resp = self
            .client
            .get(self.api_url("/memos"))
            .header("Authorization", self.auth_header())
            .query(&params)
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            401 => Err(ApiError::Unauthorized),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ApiError::Server(format!("{code}: {body}")))
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_memo(&self, name: &str) -> Result<Memo, ApiError> {
        let resp = self
            .client
            .get(self.api_url(&format!("/memos/{name}")))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            401 => Err(ApiError::Unauthorized),
            404 => Err(ApiError::NotFound),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ApiError::Server(format!("{code}: {body}")))
            }
        }
    }

    pub async fn create_memo(
        &self,
        content: &str,
        visibility: Option<&Visibility>,
    ) -> Result<Memo, ApiError> {
        let req = CreateMemoRequest {
            content: content.to_string(),
            visibility: visibility.cloned(),
            pinned: None,
        };

        let resp = self
            .client
            .post(self.api_url("/memos"))
            .header("Authorization", self.auth_header())
            .json(&req)
            .send()
            .await?;

        match resp.status().as_u16() {
            200 | 201 => Ok(resp.json().await?),
            401 => Err(ApiError::Unauthorized),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ApiError::Server(format!("{code}: {body}")))
            }
        }
    }

    pub async fn update_memo(
        &self,
        memo_id: &str,
        content: Option<&str>,
        visibility: Option<&Visibility>,
        pinned: Option<bool>,
    ) -> Result<Memo, ApiError> {
        let mut mask_fields = Vec::new();
        if content.is_some() {
            mask_fields.push("content");
        }
        if visibility.is_some() {
            mask_fields.push("visibility");
        }
        if pinned.is_some() {
            mask_fields.push("pinned");
        }
        let update_mask = mask_fields.join(",");

        let req = UpdateMemoRequest {
            content: content.map(String::from),
            visibility: visibility.cloned(),
            pinned,
            state: None,
        };

        let resp = self
            .client
            .patch(self.api_url(&format!("/memos/{memo_id}")))
            .header("Authorization", self.auth_header())
            .query(&[("updateMask", &update_mask)])
            .json(&req)
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            401 => Err(ApiError::Unauthorized),
            404 => Err(ApiError::NotFound),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ApiError::Server(format!("{code}: {body}")))
            }
        }
    }

    pub async fn delete_memo(&self, memo_id: &str) -> Result<(), ApiError> {
        let resp = self
            .client
            .delete(self.api_url(&format!("/memos/{memo_id}")))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match resp.status().as_u16() {
            200 | 204 => Ok(()),
            401 => Err(ApiError::Unauthorized),
            404 => Err(ApiError::NotFound),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ApiError::Server(format!("{code}: {body}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::Mutex;

    #[derive(Clone)]
    struct Recorded {
        method: String,
        path: String,
        query: Option<String>,
        auth: Option<String>,
        body: String,
    }

    async fn handle_conn(
        mut stream: TcpStream,
        status: u16,
        resp_body: &'static str,
        recorded: Arc<Mutex<Option<Recorded>>>,
    ) {
        let mut buf = vec![0u8; 65536];
        let mut used = 0usize;
        // Read until we have the full head + body so the client can proceed.
        loop {
            let n = stream.read(&mut buf[used..]).await.unwrap_or(0);
            if n == 0 {
                break;
            }
            used += n;
            let head_end = buf[..used].windows(4).position(|w| w == b"\r\n\r\n");
            if let Some(he) = head_end {
                let head = String::from_utf8_lossy(&buf[..he]).to_string();
                let mut lines = head.lines();
                let req_line = lines.next().unwrap_or_default();
                let mut parts = req_line.split_whitespace();
                let method = parts.next().unwrap_or("").to_string();
                let raw_target = parts.next().unwrap_or("").to_string();
                let mut auth = None;
                let mut content_len = 0usize;
                for line in lines {
                    if let Some(v) = line.strip_prefix("Authorization: ")
                        .or_else(|| line.strip_prefix("authorization: "))
                    {
                        auth = Some(v.to_string());
                    }
                    if let Some(v) = line
                        .strip_prefix("Content-Length: ")
                        .or_else(|| line.strip_prefix("content-length: "))
                    {
                        content_len = v.trim().parse().unwrap_or(0);
                    }
                }
                // Body may not have fully arrived yet; loop until we have it.
                let body_start = he + 4;
                if body_start + content_len <= used {
                    let body =
                        String::from_utf8_lossy(&buf[body_start..body_start + content_len]).to_string();
                    let (path, query) = match raw_target.split_once('?') {
                        Some((p, q)) => (p.to_string(), Some(q.to_string())),
                        None => (raw_target, None),
                    };
                    *recorded.lock().await = Some(Recorded {
                        method,
                        path,
                        query,
                        auth,
                        body,
                    });

                    let status_line = match status {
                        200 => "200 OK",
                        201 => "201 Created",
                        204 => "204 No Content",
                        401 => "401 Unauthorized",
                        404 => "404 Not Found",
                        _ => "500 Internal Server Error",
                    };
                    let resp = format!(
                        "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\n\
                         Content-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
                        resp_body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                    break;
                }
            }
        }
    }

    async fn spawn_mock(
        status: u16,
        resp_body: &'static str,
    ) -> (String, Arc<Mutex<Option<Recorded>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let recorded: Arc<Mutex<Option<Recorded>>> = Arc::new(Mutex::new(None));
        let recorded_clone = recorded.clone();
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let rec = recorded_clone.clone();
                tokio::spawn(handle_conn(stream, status, resp_body, rec));
            }
        });
        (addr, recorded)
    }

    #[tokio::test]
    async fn list_memos_builds_request_and_parses() {
        let body = r#"{"memos":[{"name":"memos/1","state":"NORMAL","creator":"users/1","create_time":"2026-01-01T10:00:00Z","update_time":"2026-01-01T10:00:01Z","content":"hello","visibility":"PRIVATE","pinned":false}],"next_page_token":"tok"}"#;
        let (addr, rec) = spawn_mock(200, body).await;
        let client = MemosClient::new(&addr, "secret");
        let resp = client.list_memos(100, "", None, "NORMAL").await.unwrap();

        assert_eq!(resp.memos.len(), 1);
        assert_eq!(resp.next_page_token, "tok");
        assert_eq!(resp.memos[0].content, "hello");
        assert_eq!(resp.memos[0].pinned, false);

        let r = rec.lock().await.clone().unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.path, "/api/v1/memos");
        assert!(r.query.unwrap().contains("state=NORMAL"));
        assert_eq!(r.auth.as_deref(), Some("Bearer secret"));
    }

    #[tokio::test]
    async fn create_memo_serializes_body() {
        let body = r#"{"name":"memos/9","content":"new memo"}"#;
        let (addr, rec) = spawn_mock(201, body).await;
        let client = MemosClient::new(&addr, "t");
        let memo = client.create_memo("new memo", None).await.unwrap();
        assert_eq!(memo.name, "memos/9");

        let r = rec.lock().await.clone().unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.path, "/api/v1/memos");
        let sent: serde_json::Value = serde_json::from_str(&r.body).unwrap();
        assert_eq!(sent["content"], "new memo");
    }

    #[tokio::test]
    async fn update_memo_builds_update_mask() {
        let body = r#"{"name":"memos/5","content":"updated","visibility":"PUBLIC"}"#;
        let (addr, rec) = spawn_mock(200, body).await;
        let client = MemosClient::new(&addr, "t");
        let vis = Visibility::Public;
        client
            .update_memo("5", Some("updated"), Some(&vis), None)
            .await
            .unwrap();

        let r = rec.lock().await.clone().unwrap();
        assert_eq!(r.method, "PATCH");
        assert_eq!(r.path, "/api/v1/memos/5");
        assert_eq!(r.query.as_deref(), Some("updateMask=content%2Cvisibility"));
        let sent: serde_json::Value = serde_json::from_str(&r.body).unwrap();
        assert_eq!(sent["content"], "updated");
        assert_eq!(sent["visibility"], "PUBLIC");
    }

    #[tokio::test]
    async fn unauthorized_maps_to_error() {
        let (addr, _) = spawn_mock(401, r#"{"message":"unauth"}"#).await;
        let client = MemosClient::new(&addr, "bad");
        match client.list_memos(10, "", None, "NORMAL").await {
            Err(ApiError::Unauthorized) => {}
            other => panic!("expected Unauthorized, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn delete_memo_maps_204_to_ok() {
        let (addr, rec) = spawn_mock(204, "").await;
        let client = MemosClient::new(&addr, "t");
        client.delete_memo("7").await.unwrap();
        let r = rec.lock().await.clone().unwrap();
        assert_eq!(r.method, "DELETE");
        assert_eq!(r.path, "/api/v1/memos/7");
    }
}