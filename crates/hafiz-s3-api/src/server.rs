//! S3 Server implementation

use axum::{
    extract::ConnectInfo,
    middleware,
    routing::{delete, get, head, post, put},
    Router,
};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hafiz_core::{config::HafizConfig, Result};
use hafiz_metadata::MetadataStore;
use hafiz_storage::LocalStorage;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tower::Service;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tower_service::Service as _;
use tracing::{error, info, warn};

use crate::routes;
use crate::admin;
use crate::metrics::{MetricsRecorder, metrics_handler, metrics_middleware};
use crate::tls::TlsAcceptor;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<HafizConfig>,
    pub storage: Arc<LocalStorage>,
    pub metadata: Arc<MetadataStore>,
    pub start_time: Instant,
    pub metrics: Arc<MetricsRecorder>,
}

/// S3 Server
pub struct S3Server {
    config: HafizConfig,
}

impl S3Server {
    pub fn new(config: HafizConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<()> {
        let start_time = Instant::now();
        
        // Validate TLS config if enabled
        if self.config.tls.enabled {
            self.config.tls.validate()?;
        }
        
        // Initialize metrics
        let metrics = Arc::new(MetricsRecorder::new());
        info!("Prometheus metrics initialized");
        
        // Initialize storage
        let storage = LocalStorage::new(&self.config.storage.data_dir);
        storage.init().await?;

        // Initialize metadata store
        let metadata = MetadataStore::new(&self.config.database.url).await?;

        // Create root user if not exists
        let root_user = hafiz_core::types::User::root(
            self.config.auth.root_access_key.clone(),
            self.config.auth.root_secret_key.clone(),
        );
        if metadata.get_user_by_access_key(&root_user.access_key).await?.is_none() {
            metadata.create_user(&root_user).await?;
            info!("Created root user with access key: {}", root_user.access_key);
        }

        let state = AppState {
            config: Arc::new(self.config.clone()),
            storage: Arc::new(storage),
            metadata: Arc::new(metadata),
            start_time,
            metrics: metrics.clone(),
        };

        let app = self.create_router(state, metrics);
        let addr = format!("{}:{}", self.config.server.bind_address, self.config.server.port);
        
        if self.config.tls.enabled {
            self.run_https(app, &addr).await
        } else {
            self.run_http(app, &addr).await
        }
    }

    async fn run_http(self, app: Router, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        
        info!("🚀 Hafiz S3 API server listening on http://{}", addr);
        info!("📊 Admin API available at http://{}/api/v1", addr);
        info!("📈 Prometheus metrics at http://{}/metrics", addr);
        info!("🔑 Access Key: {}", self.config.auth.root_access_key);
        
        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn run_https(self, app: Router, addr: &str) -> Result<()> {
        let tls_acceptor = TlsAcceptor::from_config(&self.config.tls)?;
        let listener = TcpListener::bind(addr).await?;
        
        info!("🔒 Hafiz S3 API server listening on https://{}", addr);
        info!("📊 Admin API available at https://{}/api/v1", addr);
        info!("📈 Prometheus metrics at https://{}/metrics", addr);
        info!("🔑 Access Key: {}", self.config.auth.root_access_key);
        
        if self.config.tls.require_client_cert {
            info!("🔐 mTLS enabled - client certificates required");
        }
        
        if let Some(hsts) = tls_acceptor.hsts_header() {
            info!("🛡️  HSTS enabled: {}", hsts);
        }
        
        // Log TLS version
        info!("🔒 Minimum TLS version: {:?}", self.config.tls.min_version);
        
        // Accept connections
        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };
            
            let tls_acceptor = tls_acceptor.inner().clone();
            let app = app.clone();
            
            tokio::spawn(async move {
                // Perform TLS handshake
                let tls_stream = match tls_acceptor.accept(stream).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        warn!("TLS handshake failed from {}: {}", peer_addr, e);
                        return;
                    }
                };
                
                // Create hyper service
                let io = TokioIo::new(tls_stream);
                let service = hyper::service::service_fn(move |req| {
                    let mut app = app.clone();
                    async move {
                        app.call(req).await
                    }
                });
                
                // Serve the connection
                if let Err(e) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection(io, service)
                    .await
                {
                    // Ignore connection reset errors
                    if !e.to_string().contains("connection reset") {
                        error!("Connection error from {}: {}", peer_addr, e);
                    }
                }
            });
        }
    }

    fn create_router(&self, state: AppState, metrics: Arc<MetricsRecorder>) -> Router {
        Router::new()
            // Metrics endpoint (no auth required)
            .route("/metrics", get(metrics_handler))
            
            // Admin API routes
            .nest("/api/v1", admin::admin_routes_no_auth())
            
            // Service operations
            .route("/", get(routes::list_buckets))
            
            // Bucket operations
            .route("/:bucket", head(routes::head_bucket))
            .route("/:bucket", get(routes::bucket_get_handler))  // ListObjects, ListObjectVersions, GetBucketVersioning, GetBucketLifecycle, ListMultipartUploads
            .route("/:bucket", put(routes::bucket_put_handler))  // CreateBucket, PutBucketVersioning, or PutBucketLifecycle
            .route("/:bucket", delete(routes::bucket_delete_handler)) // DeleteBucket or DeleteBucketLifecycle
            .route("/:bucket", post(routes::bucket_post_handler)) // DeleteObjects
            
            // Object operations (including multipart, versioning, and tagging)
            .route("/:bucket/*key", head(routes::head_object))
            .route("/:bucket/*key", get(routes::object_get_handler))   // GetObject, ListParts, or GetObjectTagging
            .route("/:bucket/*key", put(routes::object_put_handler))   // PutObject, CopyObject, UploadPart, or PutObjectTagging
            .route("/:bucket/*key", delete(routes::object_delete_handler)) // DeleteObject, AbortMultipart, or DeleteObjectTagging
            .route("/:bucket/*key", post(routes::object_post_handler)) // CreateMultipart or CompleteMultipart
            
            // Metrics middleware for S3 routes
            .layer(middleware::from_fn_with_state(metrics.clone(), metrics_middleware))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            )
            .layer(CorsLayer::permissive())
            .with_state(state)
    }
}
