# tonic-model-serving

## flow 
- local dev
  - sqlx to apply migrations to postgres image running in docker (so far a la zero2prod but its a nice pattern, sue me)

## goals & ideas
tangible:
- implement inference server based on gRPC 
- k8s deployment + observability to check balancing with load tests
- better telemetry
- tracing and structured logging
- health check with probing (added later in the deployment to ensure service up and running) 
- **minio** in deployment as a bucket for storing model weights
- gRPC health probe k8s 
  - there's also [this](https://github.com/grpc-ecosystem/grpc-health-probe) cli tool
  - another health endpoint would be the axum server itself
- pipe/sync logs to an elasticsearch instance ?
- [rust-cache](https://github.com/Swatinem/rust-cache) for reducing gh action times
- *graceful shutdown* for grpc and web services

- project pivot idea: use the service as an embeddings type generation thing like meilisearch and implement a database
  - could try to do an from-scratch implementation of ann
  - **or** take inspo and use thread/process pool on worker machine to handle incoming inference requests
- lightweight models would be better for testing locally; *embedding*, xgboost, time series models, <1b llms, etc.
  - need a **use case**


### notes on logging
- tracing layer for grpc like [this](https://docs.rs/tower-http/latest/tower_http/trace/struct.TraceLayer.html#method.new_for_grpc)  
  - or we can use the `interceptor` in the server init to inject the logging middleware
- we probably want 1) gRPC request-level logs, and 2) logging the internals through #\[instrument\]

conceptual:
- familiarize myself with tokio ecosystem (tonic, hyper, axum)
- review async rust
- multi-service k8s deployments (tonic grpc server, typescript(?) frontend, axum backend, db, buckets)

## todos 
- [x] [grpc basics](https://grpc.io/docs/languages/python/basics/) with examples 
- [x] skim tonic docs
- [x] add test actions and branch protection rules 
- [x] cargo workspace setup
- [x] re-read docs on streams and futures in rust (see if we can avoid the ReceiverStream pattern)
- [x] setup health probe alongside service with tonic-health [docs](https://github.com/hyperium/tonic/tree/master/examples/src/health) 
- [x] setup db to store model registry (could this be replaced down the line with mlflow?)
- [x] add reflection
- [ ] better tests
- [x] add env files serializing to app config
- [x] tracing and formatted logs
- [x] llvm linker
  - [x] read that article on minimizing build times
- [ ] add threadpool/ml inference core crate (using onnxruntime, candle, or burn)

## notes 
- to run the server and client run `cargo run --bin server` in one terminal, and `cargo run --bin client` in another
- to run a health check first spin up the server with `cargo run --bin server` then `grpc-health-probe -addr="[::1]:50051"`
- to inspect the API we can use Postman with gRPC reflection
