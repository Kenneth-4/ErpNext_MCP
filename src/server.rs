use std::sync::Arc;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool, tool_router,
};

use crate::client::ERPNextClient;
use crate::models::{
    CallMethodArgs, CancelDocumentArgs, CreateDocumentArgs, DeleteDocumentArgs,
    GetDocTypeFieldsArgs, GetDocTypesArgs, GetDocumentArgs, GetDocumentsArgs,
    RunReportArgs, SubmitDocumentArgs, UpdateDocumentArgs,
};

/// High-performance ERPNext MCP Server implementation.
#[derive(Clone, Debug)]
pub struct ERPNextServer {
    client: Arc<ERPNextClient>,
}

impl ERPNextServer {
    /// Create a new ERPNext MCP server wrapping the given client.
    pub fn new(client: ERPNextClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    /// Access the underlying Frappe/ERPNext client.
    pub fn client(&self) -> &ERPNextClient {
        &self.client
    }

    /// Helper to format API results into standard MCP CallToolResult.
    fn format_success(data: &serde_json::Value) -> CallToolResult {
        let text = serde_json::to_string_pretty(data).unwrap_or_else(|_| data.to_string());
        CallToolResult::success(vec![ContentBlock::text(text)])
    }

    /// Helper to format API errors into standard MCP CallToolResult.
    fn format_error(err: impl std::fmt::Display) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(err.to_string())])
    }
}

#[tool_router(server_handler)]
impl ERPNextServer {
    /// Retrieve list of available DocTypes from ERPNext, optionally filtered by module.
    #[tool(
        name = "get_doctypes",
        description = "Retrieve list of available DocTypes from ERPNext/Frappe. Optionally filter by module (e.g. Accounts, Stock, Selling, Buying)."
    )]
    pub async fn get_doctypes(&self, params: Parameters<GetDocTypesArgs>) -> CallToolResult {
        let args = params.0;
        match self.client.get_doctypes(args.module.as_deref()).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Get schema fields, child table definitions, and metadata for a specific DocType.
    #[tool(
        name = "get_doctype_fields",
        description = "Get schema fields, definitions, and metadata for a specific ERPNext DocType (e.g., Customer, Sales Invoice, Item)."
    )]
    pub async fn get_doctype_fields(
        &self,
        params: Parameters<GetDocTypeFieldsArgs>,
    ) -> CallToolResult {
        let args = params.0;
        match self.client.get_doctype_fields(&args.doctype).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Query list of documents for a DocType with filtering, field projection, and pagination.
    #[tool(
        name = "get_documents",
        description = "Query a list of documents for a DocType with optional filters, field projection, limit, and ordering."
    )]
    pub async fn get_documents(&self, params: Parameters<GetDocumentsArgs>) -> CallToolResult {
        let args = params.0;
        match self
            .client
            .get_documents(
                &args.doctype,
                args.filters.as_ref(),
                args.fields.as_deref(),
                args.limit,
                args.order_by.as_deref(),
            )
            .await
        {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Fetch a single document by name, including all child tables and details.
    #[tool(
        name = "get_document",
        description = "Fetch a single document by DocType and unique document name, including child tables."
    )]
    pub async fn get_document(&self, params: Parameters<GetDocumentArgs>) -> CallToolResult {
        let args = params.0;
        match self.client.get_document(&args.doctype, &args.name).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Create a new document in ERPNext.
    #[tool(
        name = "create_document",
        description = "Create a new document in ERPNext for the specified DocType with the provided field data."
    )]
    pub async fn create_document(&self, params: Parameters<CreateDocumentArgs>) -> CallToolResult {
        let args = params.0;
        match self.client.create_document(&args.doctype, args.data).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Update fields of an existing document.
    #[tool(
        name = "update_document",
        description = "Update fields of an existing document in ERPNext by DocType and document name."
    )]
    pub async fn update_document(&self, params: Parameters<UpdateDocumentArgs>) -> CallToolResult {
        let args = params.0;
        match self
            .client
            .update_document(&args.doctype, &args.name, args.data)
            .await
        {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Permanently delete a document.
    #[tool(
        name = "delete_document",
        description = "Permanently delete an ERPNext document by DocType and document name."
    )]
    pub async fn delete_document(&self, params: Parameters<DeleteDocumentArgs>) -> CallToolResult {
        let args = params.0;
        match self.client.delete_document(&args.doctype, &args.name).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Submit a submittable document (sets docstatus to 1).
    #[tool(
        name = "submit_document",
        description = "Submit a submittable ERPNext document (e.g. Sales Invoice, Purchase Order), changing its docstatus to 1."
    )]
    pub async fn submit_document(&self, params: Parameters<SubmitDocumentArgs>) -> CallToolResult {
        let args = params.0;
        match self.client.submit_document(&args.doctype, &args.name).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Cancel a submitted document (sets docstatus to 2).
    #[tool(
        name = "cancel_document",
        description = "Cancel a submitted ERPNext document (e.g. Sales Invoice, Purchase Order), changing its docstatus to 2."
    )]
    pub async fn cancel_document(&self, params: Parameters<CancelDocumentArgs>) -> CallToolResult {
        let args = params.0;
        match self.client.cancel_document(&args.doctype, &args.name).await {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Execute any whitelisted Frappe Python server-side method.
    #[tool(
        name = "call_method",
        description = "Execute a whitelisted Frappe Python server-side method with optional arguments via POST or GET."
    )]
    pub async fn call_method(&self, params: Parameters<CallMethodArgs>) -> CallToolResult {
        let args = params.0;
        match self
            .client
            .call_method(&args.method, args.args, args.http_method.as_deref())
            .await
        {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }

    /// Run standard or script ERPNext reports.
    #[tool(
        name = "run_report",
        description = "Run standard or script ERPNext reports (e.g. General Ledger, Accounts Receivable Summary) with optional filter parameters."
    )]
    pub async fn run_report(&self, params: Parameters<RunReportArgs>) -> CallToolResult {
        let args = params.0;
        match self
            .client
            .run_report(&args.report_name, args.filters.as_ref())
            .await
        {
            Ok(data) => Self::format_success(&data),
            Err(e) => Self::format_error(e),
        }
    }
}
