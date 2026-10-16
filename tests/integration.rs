use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use erpnext_mcp::models::*;
use erpnext_mcp::{ERPNextClient, ERPNextServer};

#[tokio::test]
async fn test_get_doctypes_without_filter() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/resource/DocType"))
        .and(query_param("fields", "[\"name\",\"module\",\"istable\"]"))
        .and(query_param("limit_page_length", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "name": "Customer", "module": "Selling", "istable": 0 },
                { "name": "Sales Invoice", "module": "Accounts", "istable": 0 }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .get_doctypes(Parameters(GetDocTypesArgs { module: None }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("Customer"));
    assert!(text.contains("Sales Invoice"));
}

#[tokio::test]
async fn test_get_doctypes_with_module_filter() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/resource/DocType"))
        .and(query_param("limit_page_length", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "name": "Sales Invoice", "module": "Accounts", "istable": 0 }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .get_doctypes(Parameters(GetDocTypesArgs {
            module: Some("Accounts".into()),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("Sales Invoice"));
}

#[tokio::test]
async fn test_get_doctype_fields_with_url_encoding() {
    let mock_server = MockServer::start().await;

    // DocType with a space must be URL-encoded as "Sales%20Invoice"
    Mock::given(method("GET"))
        .and(path("/api/resource/DocType/Sales%20Invoice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "name": "Sales Invoice",
                "module": "Accounts",
                "fields": [
                    { "fieldname": "customer", "label": "Customer", "fieldtype": "Link", "reqd": 1 },
                    { "fieldname": "posting_date", "label": "Date", "fieldtype": "Date", "reqd": 1 }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .get_doctype_fields(Parameters(GetDocTypeFieldsArgs {
            doctype: "Sales Invoice".into(),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("customer"));
    assert!(text.contains("posting_date"));
}

#[tokio::test]
async fn test_get_documents_with_query_params() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/resource/Customer"))
        .and(query_param("limit_page_length", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "name": "CUST-00001", "customer_name": "Acme Industrial" },
                { "name": "CUST-00002", "customer_name": "Globex Corp" }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .get_documents(Parameters(GetDocumentsArgs {
            doctype: "Customer".into(),
            filters: Some(json!({"status": "Active"})),
            fields: Some(vec!["name".into(), "customer_name".into()]),
            limit: Some(5),
            order_by: Some("creation desc".into()),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("CUST-00001"));
    assert!(text.contains("Acme Industrial"));
}

#[tokio::test]
async fn test_get_document_with_special_characters() {
    let mock_server = MockServer::start().await;

    // ERPNext document name with special characters (e.g., ACC-SINV-.2026.-0001)
    Mock::given(method("GET"))
        .and(path("/api/resource/Sales%20Invoice/ACC-SINV-.2026.-0001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "name": "ACC-SINV-.2026.-0001",
                "customer": "CUST-00001",
                "grand_total": 1500.50,
                "docstatus": 0,
                "items": [
                    { "item_code": "ITEM-A", "qty": 2, "rate": 750.25 }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .get_document(Parameters(GetDocumentArgs {
            doctype: "Sales Invoice".into(),
            name: "ACC-SINV-.2026.-0001".into(),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("ACC-SINV-.2026.-0001"));
    assert!(text.contains("1500.5"));
}

#[tokio::test]
async fn test_create_document() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/resource/Customer"))
        .and(body_json(json!({
            "data": {
                "customer_name": "Cyberdyne Systems",
                "customer_type": "Company"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "name": "CUST-00099",
                "customer_name": "Cyberdyne Systems",
                "customer_type": "Company"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .create_document(Parameters(CreateDocumentArgs {
            doctype: "Customer".into(),
            data: json!({
                "customer_name": "Cyberdyne Systems",
                "customer_type": "Company"
            }),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("CUST-00099"));
}

#[tokio::test]
async fn test_update_document() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/resource/Customer/CUST-00099"))
        .and(body_json(json!({
            "data": {
                "customer_name": "Cyberdyne Systems LLC"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "name": "CUST-00099",
                "customer_name": "Cyberdyne Systems LLC"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .update_document(Parameters(UpdateDocumentArgs {
            doctype: "Customer".into(),
            name: "CUST-00099".into(),
            data: json!({ "customer_name": "Cyberdyne Systems LLC" }),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("Cyberdyne Systems LLC"));
}

#[tokio::test]
async fn test_delete_document() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/resource/Customer/CUST-00099"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .delete_document(Parameters(DeleteDocumentArgs {
            doctype: "Customer".into(),
            name: "CUST-00099".into(),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
}

#[tokio::test]
async fn test_submit_document() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/resource/Sales%20Invoice/ACC-SINV-001"))
        .and(body_json(json!({
            "data": {
                "docstatus": 1
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "name": "ACC-SINV-001",
                "docstatus": 1
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .submit_document(Parameters(SubmitDocumentArgs {
            doctype: "Sales Invoice".into(),
            name: "ACC-SINV-001".into(),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("\"docstatus\": 1"));
}

#[tokio::test]
async fn test_cancel_document() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/resource/Sales%20Invoice/ACC-SINV-001"))
        .and(body_json(json!({
            "data": {
                "docstatus": 2
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "name": "ACC-SINV-001",
                "docstatus": 2
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .cancel_document(Parameters(CancelDocumentArgs {
            doctype: "Sales Invoice".into(),
            name: "ACC-SINV-001".into(),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("\"docstatus\": 2"));
}

#[tokio::test]
async fn test_call_method() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/method/frappe.client.get_value"))
        .and(body_json(json!({
            "doctype": "Customer",
            "fieldname": "customer_name",
            "filters": { "name": "CUST-00001" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": { "customer_name": "Acme Industrial" }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .call_method(Parameters(CallMethodArgs {
            method: "frappe.client.get_value".into(),
            args: Some(json!({
                "doctype": "Customer",
                "fieldname": "customer_name",
                "filters": { "name": "CUST-00001" }
            })),
            http_method: Some("POST".into()),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("Acme Industrial"));
}

#[tokio::test]
async fn test_run_report() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/method/frappe.desk.query_report.run"))
        .and(query_param("report_name", "General Ledger"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": {
                "columns": [
                    { "label": "Posting Date", "fieldname": "posting_date", "fieldtype": "Date" },
                    { "label": "Account", "fieldname": "account", "fieldtype": "Link" }
                ],
                "result": [
                    { "posting_date": "2026-01-01", "account": "1110 - Cash - HTI", "debit": 1000 }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .run_report(Parameters(RunReportArgs {
            report_name: "General Ledger".into(),
            filters: Some(json!({"company": "HTI Corp"})),
        }))
        .await;

    assert_eq!(res.is_error, Some(false));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("1110 - Cash - HTI"));
    assert!(text.contains("posting_date"));
}

#[tokio::test]
async fn test_frappe_error_extraction_server_messages() {
    let mock_server = MockServer::start().await;

    // Frappe returns errors inside _server_messages formatted as stringified JSON array
    let server_messages = serde_json::to_string(&vec![
        json!({"message": "<strong>Customer</strong> is mandatory field"}).to_string(),
    ])
    .unwrap();

    Mock::given(method("POST"))
        .and(path("/api/resource/Sales%20Invoice"))
        .respond_with(
            ResponseTemplate::new(417).set_body_json(json!({
                "exc_type": "ValidationError",
                "_server_messages": server_messages
            })),
        )
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(&mock_server.uri(), None, None).unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .create_document(Parameters(CreateDocumentArgs {
            doctype: "Sales Invoice".into(),
            data: json!({ "posting_date": "2026-01-01" }),
        }))
        .await;

    assert_eq!(res.is_error, Some(true));
    let text = res.content[0].as_text().unwrap().text.as_str();
    assert!(text.contains("Customer is mandatory field"));
    // HTML tags should be stripped
    assert!(!text.contains("<strong>"));
}

#[tokio::test]
async fn test_auth_headers_attached() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/resource/DocType"))
        .and(header("Authorization", "token testkey123:testsecret456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [] })))
        .mount(&mock_server)
        .await;

    let client = ERPNextClient::new(
        &mock_server.uri(),
        Some("testkey123"),
        Some("testsecret456"),
    )
    .unwrap();
    let server = ERPNextServer::new(client);

    let res = server
        .get_doctypes(Parameters(GetDocTypesArgs { module: None }))
        .await;

    assert_eq!(res.is_error, Some(false));
}
