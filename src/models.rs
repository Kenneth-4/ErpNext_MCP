use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Arguments for `get_doctypes` tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GetDocTypesArgs {
    /// Optional Frappe/ERPNext module name to filter DocTypes (e.g., "Accounts", "Stock", "Buying", "Selling").
    #[serde(default)]
    pub module: Option<String>,
}

/// Arguments for `get_doctype_fields` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetDocTypeFieldsArgs {
    /// The name of the DocType to inspect (e.g., "Customer", "Sales Invoice", "Item").
    pub doctype: String,
}

/// Arguments for `get_documents` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetDocumentsArgs {
    /// Name of the DocType to query (e.g., "Customer", "Sales Invoice", "Item").
    pub doctype: String,

    /// Optional filter criteria. Can be a key-value object (e.g., `{"status": "Draft", "customer_group": "Commercial"}`)
    /// or a Frappe filter array (e.g., `[["status", "=", "Draft"]]`).
    #[serde(default)]
    pub filters: Option<serde_json::Value>,

    /// Optional list of specific field names to retrieve (e.g., `["name", "customer_name", "status"]`).
    #[serde(default)]
    pub fields: Option<Vec<String>>,

    /// Maximum number of documents to return (default: 20).
    #[serde(default)]
    pub limit: Option<u32>,

    /// Sorting criteria for the returned records (e.g., "creation desc", "name asc").
    #[serde(default)]
    pub order_by: Option<String>,
}

/// Arguments for `get_document` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetDocumentArgs {
    /// Name of the DocType (e.g., "Customer", "Sales Invoice").
    pub doctype: String,

    /// Unique document identifier/name (e.g., "CUST-00001", "ACC-SINV-.2026.-0001").
    pub name: String,
}

/// Arguments for `create_document` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocumentArgs {
    /// Name of the DocType to create (e.g., "Customer", "Task", "Lead").
    pub doctype: String,

    /// JSON object containing the field names and values for the new document.
    pub data: serde_json::Value,
}

/// Arguments for `update_document` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateDocumentArgs {
    /// Name of the DocType (e.g., "Customer", "Sales Invoice").
    pub doctype: String,

    /// Unique identifier/name of the document to update.
    pub name: String,

    /// JSON object containing updated field values.
    pub data: serde_json::Value,
}

/// Arguments for `delete_document` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteDocumentArgs {
    /// Name of the DocType (e.g., "Customer", "Task").
    pub doctype: String,

    /// Unique identifier/name of the document to delete.
    pub name: String,
}

/// Arguments for `submit_document` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubmitDocumentArgs {
    /// Name of the submittable DocType (e.g., "Sales Invoice", "Purchase Order").
    pub doctype: String,

    /// Unique identifier/name of the document to submit (sets docstatus to 1).
    pub name: String,
}

/// Arguments for `cancel_document` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelDocumentArgs {
    /// Name of the submitted DocType (e.g., "Sales Invoice", "Purchase Order").
    pub doctype: String,

    /// Unique identifier/name of the document to cancel (sets docstatus to 2).
    pub name: String,
}

/// Arguments for `call_method` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CallMethodArgs {
    /// Dotted Python path to the whitelisted Frappe method (e.g., "frappe.client.get_value", "frappe.desk.form.load.getdoctype").
    pub method: String,

    /// Optional argument payload to pass to the method.
    #[serde(default)]
    pub args: Option<serde_json::Value>,

    /// HTTP request method: "POST" (default) or "GET".
    #[serde(default)]
    pub http_method: Option<String>,
}

/// Arguments for `run_report` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunReportArgs {
    /// Name of the report to run (e.g., "General Ledger", "Accounts Receivable Summary", "Stock Balance").
    pub report_name: String,

    /// Optional filter parameters for the report as a JSON object (e.g., `{"company": "Acme Corp", "fiscal_year": "2026"}`).
    #[serde(default)]
    pub filters: Option<serde_json::Value>,
}
