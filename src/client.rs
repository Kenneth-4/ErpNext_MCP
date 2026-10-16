use std::time::Duration;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Value, json};
use thiserror::Error;

/// ERPNext and Frappe API client errors.
#[derive(Debug, Error)]
pub enum ERPNextError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("ERPNext API error: {0}")]
    Api(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Frappe/ERPNext REST API client.
#[derive(Clone, Debug)]
pub struct ERPNextClient {
    client: reqwest::Client,
    base_url: String,
    auth_header: Option<HeaderValue>,
}

impl ERPNextClient {
    /// Create a new ERPNext API client.
    ///
    /// - `base_url`: Target ERPNext or Frappe instance URL (e.g. `https://erp.example.com`).
    /// - `api_key`: Optional Frappe API key.
    /// - `api_secret`: Optional Frappe API secret.
    pub fn new(
        base_url: &str,
        api_key: Option<&str>,
        api_secret: Option<&str>,
    ) -> Result<Self, ERPNextError> {
        let trimmed_url = base_url.trim().trim_end_matches('/').to_string();
        if trimmed_url.is_empty() {
            return Err(ERPNextError::Config(
                "ERPNEXT_URL must not be empty".to_string(),
            ));
        }

        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let auth_header = match (api_key, api_secret) {
            (Some(key), Some(secret)) if !key.trim().is_empty() && !secret.trim().is_empty() => {
                let token_value = format!("token {}:{}", key.trim(), secret.trim());
                let header_val = HeaderValue::from_str(&token_value).map_err(|e| {
                    ERPNextError::Config(format!("Invalid characters in API key/secret: {e}"))
                })?;
                Some(header_val)
            }
            _ => None,
        };

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: trimmed_url,
            auth_header,
        })
    }

    /// Base URL of the configured ERPNext instance.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Whether this client is configured with authentication credentials.
    pub fn is_authenticated(&self) -> bool {
        self.auth_header.is_some()
    }

    /// Internal helper to execute an HTTP request and inspect Frappe responses.
    async fn execute_request(
        &self,
        mut req_builder: reqwest::RequestBuilder,
    ) -> Result<Value, ERPNextError> {
        if let Some(ref auth) = self.auth_header {
            req_builder = req_builder.header(AUTHORIZATION, auth.clone());
        }

        let response = req_builder.send().await?;
        let status = response.status();
        let body_bytes = response.bytes().await?;
        let body_text = String::from_utf8_lossy(&body_bytes);

        if !status.is_success() {
            let error_msg = extract_frappe_error(status, &body_text);
            return Err(ERPNextError::Api(error_msg));
        }

        if body_text.trim().is_empty() {
            return Ok(json!({ "status": "ok" }));
        }

        let json_val: Value = serde_json::from_str(&body_text).map_err(|e| {
            ERPNextError::Api(format!(
                "Failed to parse ERPNext response as JSON: {e}. Raw response: {body_text}"
            ))
        })?;

        Ok(unwrap_frappe_payload(json_val))
    }

    /// 1. `get_doctypes`: Retrieve list of available DocTypes.
    pub async fn get_doctypes(&self, module: Option<&str>) -> Result<Value, ERPNextError> {
        let mut url = format!(
            "{}/api/resource/DocType?fields=[\"name\",\"module\",\"istable\"]&limit_page_length=0",
            self.base_url
        );

        if let Some(m) = module {
            let filter_json = json!([["module", "=", m]]);
            url.push_str("&filters=");
            url.push_str(&urlencoding::encode(&filter_json.to_string()));
        }

        let req = self.client.get(&url);
        self.execute_request(req).await
    }

    /// 2. `get_doctype_fields`: Get schema fields and metadata for a specific DocType.
    pub async fn get_doctype_fields(&self, doctype: &str) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let url = format!("{}/api/resource/DocType/{}", self.base_url, encoded_doctype);
        let req = self.client.get(&url);
        self.execute_request(req).await
    }

    /// 3. `get_documents`: Query list of documents for a DocType with filtering, field projection, and pagination.
    pub async fn get_documents(
        &self,
        doctype: &str,
        filters: Option<&Value>,
        fields: Option<&[String]>,
        limit: Option<u32>,
        order_by: Option<&str>,
    ) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let mut url = format!("{}/api/resource/{}", self.base_url, encoded_doctype);

        let page_limit = limit.unwrap_or(20);
        url.push_str(&format!("?limit_page_length={page_limit}"));

        if let Some(f) = filters {
            url.push_str("&filters=");
            url.push_str(&urlencoding::encode(&f.to_string()));
        }

        if let Some(flds) = fields {
            let fields_json = serde_json::to_string(flds)?;
            url.push_str("&fields=");
            url.push_str(&urlencoding::encode(&fields_json));
        }

        if let Some(order) = order_by {
            url.push_str("&order_by=");
            url.push_str(&urlencoding::encode(order));
        }

        let req = self.client.get(&url);
        self.execute_request(req).await
    }

    /// 4. `get_document`: Fetch a single document by name, including child tables.
    pub async fn get_document(&self, doctype: &str, name: &str) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/api/resource/{}/{}",
            self.base_url, encoded_doctype, encoded_name
        );
        let req = self.client.get(&url);
        self.execute_request(req).await
    }

    /// 5. `create_document`: Create a new document in ERPNext.
    pub async fn create_document(&self, doctype: &str, data: Value) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let url = format!("{}/api/resource/{}", self.base_url, encoded_doctype);

        // Ensure payload is formatted as {"data": data}
        let payload = if data.is_object() && data.get("data").is_some() {
            data
        } else {
            json!({ "data": data })
        };

        let req = self.client.post(&url).json(&payload);
        self.execute_request(req).await
    }

    /// 6. `update_document`: Update fields of an existing document.
    pub async fn update_document(
        &self,
        doctype: &str,
        name: &str,
        data: Value,
    ) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/api/resource/{}/{}",
            self.base_url, encoded_doctype, encoded_name
        );

        let payload = if data.is_object() && data.get("data").is_some() {
            data
        } else {
            json!({ "data": data })
        };

        let req = self.client.put(&url).json(&payload);
        self.execute_request(req).await
    }

    /// 7. `delete_document`: Permanently delete a document.
    pub async fn delete_document(&self, doctype: &str, name: &str) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/api/resource/{}/{}",
            self.base_url, encoded_doctype, encoded_name
        );
        let req = self.client.delete(&url);
        self.execute_request(req).await
    }

    /// 8. `submit_document`: Submit a submittable document (sets docstatus to 1).
    pub async fn submit_document(&self, doctype: &str, name: &str) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/api/resource/{}/{}",
            self.base_url, encoded_doctype, encoded_name
        );
        let payload = json!({
            "data": {
                "docstatus": 1
            }
        });
        let req = self.client.put(&url).json(&payload);
        self.execute_request(req).await
    }

    /// 9. `cancel_document`: Cancel a submitted document (sets docstatus to 2).
    pub async fn cancel_document(&self, doctype: &str, name: &str) -> Result<Value, ERPNextError> {
        let encoded_doctype = urlencoding::encode(doctype);
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/api/resource/{}/{}",
            self.base_url, encoded_doctype, encoded_name
        );
        let payload = json!({
            "data": {
                "docstatus": 2
            }
        });
        let req = self.client.put(&url).json(&payload);
        self.execute_request(req).await
    }

    /// 10. `call_method`: Execute any whitelisted Frappe Python server-side method.
    pub async fn call_method(
        &self,
        method: &str,
        args: Option<Value>,
        http_method: Option<&str>,
    ) -> Result<Value, ERPNextError> {
        let encoded_method = method
            .split('.')
            .map(|segment| urlencoding::encode(segment).into_owned())
            .collect::<Vec<_>>()
            .join(".");

        let is_get = http_method
            .map(|m| m.eq_ignore_ascii_case("GET"))
            .unwrap_or(false);

        if is_get {
            let mut url = format!("{}/api/method/{}", self.base_url, encoded_method);
            if let Some(Value::Object(map)) = args {
                let mut first = true;
                for (k, v) in map {
                    let sep = if first {
                        first = false;
                        '?'
                    } else {
                        '&'
                    };
                    let val_str = match v {
                        Value::String(s) => s,
                        other => other.to_string(),
                    };
                    url.push(sep);
                    url.push_str(&urlencoding::encode(&k));
                    url.push('=');
                    url.push_str(&urlencoding::encode(&val_str));
                }
            }
            let req = self.client.get(&url);
            self.execute_request(req).await
        } else {
            let url = format!("{}/api/method/{}", self.base_url, encoded_method);
            let payload = args.unwrap_or_else(|| json!({}));
            let req = self.client.post(&url).json(&payload);
            self.execute_request(req).await
        }
    }

    /// 11. `run_report`: Run standard or script ERPNext reports.
    pub async fn run_report(
        &self,
        report_name: &str,
        filters: Option<&Value>,
    ) -> Result<Value, ERPNextError> {
        let mut url = format!(
            "{}/api/method/frappe.desk.query_report.run?report_name={}",
            self.base_url,
            urlencoding::encode(report_name)
        );

        if let Some(f) = filters {
            url.push_str("&filters=");
            url.push_str(&urlencoding::encode(&f.to_string()));
        }

        let req = self.client.get(&url);
        self.execute_request(req).await
    }
}

/// Unwraps Frappe's standard {"data": ...} or {"message": ...} envelope.
pub fn unwrap_frappe_payload(val: Value) -> Value {
    if let Value::Object(ref map) = val {
        if let Some(data) = map.get("data") {
            return data.clone();
        }
        if let Some(msg) = map.get("message") {
            return msg.clone();
        }
    }
    val
}

/// Robustly extracts clean, human-readable error messages from Frappe JSON error payloads.
pub fn extract_frappe_error(status: reqwest::StatusCode, body_text: &str) -> String {
    if let Ok(val) = serde_json::from_str::<Value>(body_text) {
        // 1. Inspect `_server_messages`
        // In Frappe, this is typically a JSON-encoded string containing an array of serialized message objects.
        if let Some(server_messages) = val.get("_server_messages") {
            let parsed_messages: Option<Vec<Value>> = if let Some(s) = server_messages.as_str() {
                serde_json::from_str::<Vec<Value>>(s).ok()
            } else if let Some(arr) = server_messages.as_array() {
                Some(arr.clone())
            } else {
                None
            };

            if let Some(msgs) = parsed_messages {
                let mut extracted = Vec::new();
                for m in msgs {
                    if let Some(msg_str) = m.as_str() {
                        if let Ok(inner) = serde_json::from_str::<Value>(msg_str) {
                            if let Some(text) = inner.get("message").and_then(|t| t.as_str()) {
                                extracted.push(strip_html(text));
                                continue;
                            }
                        }
                        extracted.push(strip_html(msg_str));
                    } else if let Some(obj) = m.as_object() {
                        if let Some(text) = obj.get("message").and_then(|t| t.as_str()) {
                            extracted.push(strip_html(text));
                        }
                    }
                }
                let clean = extracted
                    .into_iter()
                    .filter(|s| !s.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                if !clean.is_empty() {
                    return clean;
                }
            }
        }

        // 2. Inspect `exception` (e.g. "frappe.exceptions.DoesNotExistError: DocType XYZ not found")
        if let Some(exc) = val.get("exception").and_then(|e| e.as_str()) {
            let clean = exc
                .lines()
                .filter(|line| !line.trim().is_empty())
                .last()
                .unwrap_or(exc)
                .trim();
            if !clean.is_empty() {
                return clean.to_string();
            }
        }

        // 3. Inspect `message`
        if let Some(msg) = val.get("message") {
            if let Some(msg_str) = msg.as_str() {
                let clean = strip_html(msg_str);
                if !clean.trim().is_empty() {
                    return clean;
                }
            } else if !msg.is_null() {
                return msg.to_string();
            }
        }

        // 4. Inspect `error` or `errors`
        if let Some(err) = val.get("error").and_then(|e| e.as_str()) {
            let clean = strip_html(err);
            if !clean.trim().is_empty() {
                return clean;
            }
        }
    }

    // Fallback: clean up non-JSON body or HTTP status
    let trimmed = body_text.trim();
    if !trimmed.is_empty() && trimmed.len() < 300 {
        format!("HTTP {} error: {}", status.as_u16(), strip_html(trimmed))
    } else {
        format!("HTTP error {}", status)
    }
}

/// Simple helper to strip HTML tags (<p>, <div>, etc.) commonly returned in Frappe errors.
fn strip_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;

    for c in input.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(c),
            _ => {}
        }
    }

    output.trim().to_string()
}
