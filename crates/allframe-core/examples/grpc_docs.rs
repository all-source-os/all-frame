//! gRPC Service Explorer Example
//!
//! This example demonstrates how to use AllFrame's gRPC Explorer integration
//! to generate beautiful, interactive gRPC API documentation.
//!
//! Features demonstrated:
//! - gRPC Explorer configuration
//! - Multiple theme options
//! - gRPC reflection support
//! - TLS/SSL configuration
//! - Custom headers configuration
//! - Timeout management
//!
//! Run with: cargo run --example grpc_docs --features "router,openapi"

use allframe_core::router::{grpc_explorer_html, GrpcExplorerConfig, GrpcExplorerTheme, Router};

fn main() {
    println!("🚀 AllFrame gRPC Documentation Example");
    println!("========================================\n");

    // Create router (in a real app, this would have gRPC handlers)
    let _router = Router::new();
    println!("✅ Router created\n");

    // Configure gRPC Explorer with all features
    println!("🎨 Configuring gRPC Explorer...");

    let grpc_config = GrpcExplorerConfig::new()
        .server_url("http://localhost:50051")
        .enable_reflection(true)
        .enable_tls(false)
        .theme(GrpcExplorerTheme::Dark)
        .timeout_seconds(30)
        .add_header("Authorization", "Bearer your-token-here")
        .add_header("X-API-Version", "v1")
        .custom_css(
            r#"
            body {
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            }
            .grpc-explorer {
                --color-primary: 96, 165, 250;
            }
        "#,
        );

    println!("   • Server: http://localhost:50051");
    println!("   • Reflection: Enabled");
    println!("   • TLS: Disabled (development mode)");
    println!("   • Theme: Dark mode");
    println!("   • Timeout: 30 seconds");
    println!("   • Custom headers: 2 configured");
    println!("   • Custom CSS: Applied\n");

    // Generate gRPC Explorer HTML
    println!("🎭 Generating gRPC Explorer HTML documentation...");

    let grpc_html = grpc_explorer_html(&grpc_config, "AllFrame gRPC API");

    println!("✅ gRPC Explorer HTML generated ({} bytes)\n", grpc_html.len());

    // Show what the HTML contains
    println!("📦 Generated documentation includes:");
    println!("   ✅ Interactive gRPC service explorer");
    println!("   ✅ Service and method browser");
    println!("   ✅ Request builder with JSON input");
    println!("   ✅ Real-time request/response testing");
    println!("   ✅ gRPC reflection support");
    println!("   ✅ Stream testing (server/client/bidirectional)");
    println!("   ✅ Dark theme by default");
    println!();

    // Usage instructions
    println!("🚀 Next Steps:");
    println!("   1. Integrate with your web framework (Axum, Actix, etc.)");
    println!("   2. Serve the HTML at /grpc/explorer");
    println!("   3. Implement gRPC server with reflection");
    println!("   4. Start testing your gRPC services!");
    println!();

    // Example integration code
    println!("💡 Example Integration (Axum with Tonic):");
    println!(
        r#"
    use axum::{{routing::get, Router, response::Html}};
    use tonic::{{transport::Server, Request, Response, Status}};
    use tonic_reflection::server::Builder;

    // Define your gRPC service
    pub mod hello {{
        tonic::include_proto!("hello");
    }}

    use hello::greeter_server::{{Greeter, GreeterServer}};
    use hello::{{HelloRequest, HelloReply}};

    #[derive(Default)]
    pub struct MyGreeter {{}}

    #[tonic::async_trait]
    impl Greeter for MyGreeter {{
        async fn say_hello(
            &self,
            request: Request<HelloRequest>,
        ) -> Result<Response<HelloReply>, Status> {{
            let name = request.into_inner().name;

            Ok(Response::new(HelloReply {{
                message: format!("Hello {{}}!", name),
            }}))
        }}
    }}

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {{
        // Start gRPC server with reflection on port 50051
        let grpc_addr = "0.0.0.0:50051".parse()?;

        let greeter = MyGreeter::default();
        let service = GreeterServer::new(greeter);

        // Enable gRPC reflection
        let reflection_service = Builder::configure()
            .register_encoded_file_descriptor_set(
                hello::FILE_DESCRIPTOR_SET
            )
            .build()?;

        tokio::spawn(async move {{
            Server::builder()
                .add_service(reflection_service)
                .add_service(service)
                .serve(grpc_addr)
                .await
                .unwrap();
        }});

        println!("🔌 gRPC Server: http://localhost:50051");

        // Start HTTP server for gRPC Explorer on port 3000
        let app = Router::new()
            .route("/grpc/explorer", get(|| async {{
                Html(grpc_html)
            }}));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await?;

        println!("📚 gRPC Explorer: http://localhost:3000/grpc/explorer");

        axum::serve(listener, app).await?;

        Ok(())
    }}
    "#
    );

    // Example proto file
    println!("\n💡 Example Proto File (hello.proto):");
    println!(
        r#"
    syntax = "proto3";

    package hello;

    // The greeting service definition
    service Greeter {{
        // Sends a greeting
        rpc SayHello (HelloRequest) returns (HelloReply) {{}}

        // Server streaming
        rpc SayHelloStream (HelloRequest) returns (stream HelloReply) {{}}

        // Client streaming
        rpc SayHelloClientStream (stream HelloRequest) returns (HelloReply) {{}}

        // Bidirectional streaming
        rpc SayHelloBidirectional (stream HelloRequest) returns (stream HelloReply) {{}}
    }}

    // The request message
    message HelloRequest {{
        string name = 1;
    }}

    // The response message
    message HelloReply {{
        string message = 1;
    }}
    "#
    );

    // Example gRPC requests
    println!("\n💡 Example gRPC Requests to Try:");
    println!(
        r#"
    // Unary call
    {{
        "name": "World"
    }}

    // Server streaming - same request, multiple responses
    {{
        "name": "Alice"
    }}

    // Client streaming - multiple requests, single response
    [
        {{ "name": "Bob" }},
        {{ "name": "Charlie" }},
        {{ "name": "David" }}
    ]

    // Bidirectional streaming - multiple requests and responses
    [
        {{ "name": "User1" }},
        {{ "name": "User2" }}
    ]
    "#
    );

    // gRPC Reflection setup
    println!("\n📋 Setting up gRPC Reflection:");
    println!("   1. Add tonic-reflection to Cargo.toml:");
    println!("      tonic-reflection = \"0.11\"");
    println!();
    println!("   2. Include file descriptor set in build.rs:");
    println!(
        r#"
      tonic_build::configure()
          .file_descriptor_set_path("proto_descriptor.bin")
          .compile(&["proto/hello.proto"], &["proto"])?;
    "#
    );
    println!();
    println!("   3. Include descriptor set in your service:");
    println!(
        r#"
      const FILE_DESCRIPTOR_SET: &[u8] =
          include_bytes!("../proto_descriptor.bin");
    "#
    );

    // Configuration options
    println!("\n⚙️ Available Configuration Options:");
    println!("   • server_url: gRPC server URL");
    println!("   • enable_reflection: Enable service discovery");
    println!("   • enable_tls: Enable TLS/SSL");
    println!("   • theme: Light or Dark");
    println!("   • timeout_seconds: Request timeout");
    println!("   • add_header: Add custom metadata headers");
    println!("   • custom_css: Inject custom styling");
    println!();

    // Feature comparison
    println!("📊 gRPC Explorer Features:");
    println!("   Feature               | Status");
    println!("   ─────────────────────────────");
    println!("   Service Discovery     | ✓ (via reflection)");
    println!("   Unary Calls          | ✓");
    println!("   Server Streaming     | ✓");
    println!("   Client Streaming     | ✓");
    println!("   Bidirectional        | ✓");
    println!("   Custom Metadata      | ✓");
    println!("   TLS Support          | ✓");
    println!("   Dark Mode            | ✓");
    println!();

    // Troubleshooting
    println!("🔧 Troubleshooting:");
    println!("   Problem: Can't connect to gRPC server");
    println!("   Solution: Ensure server is running on specified URL");
    println!();
    println!("   Problem: No services showing up");
    println!("   Solution: Enable gRPC reflection in your server");
    println!();
    println!("   Problem: CORS errors in browser");
    println!("   Solution: Use gRPC-Web proxy (e.g., Envoy)");
    println!();

    println!("✨ Example complete!");
    println!("\n🎯 Key Takeaways:");
    println!("   1. gRPC Explorer provides interactive gRPC documentation");
    println!("   2. Reflection enables automatic service discovery");
    println!("   3. Supports all gRPC call types (unary, streaming)");
    println!("   4. Customizable theming and styling");
    println!("   5. Framework-agnostic - works with any Rust web framework");
    println!("   6. Production-ready with TLS and authentication support");
}
