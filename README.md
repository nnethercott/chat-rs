# tonic-model-serving

## goals & ideas
tangible:
* implement inference server based on gRPC 
* leverage cargo workspaces 
  * at least two crates - one for front and one for back
* k8s deployment + observability to check balancing with load tests
* better telemetry
* tracing and structured logging
* health check with probing (added later in the deployment to ensure service up and running) 

conceptual:
* familiarize myself with tokio ecosystem 
* review async rust
* multi-service k8s deployments

## todos 
- [x] [grpc basics](https://grpc.io/docs/languages/python/basics/) with examples 
- [x] skim tonic docs
- [x] add test actions and branch protection rules 
- [x] cargo workspace setup
- [x] re-read docs on streams and futures in rust (see if we can avoid the ReceiverStream pattern)
- [ ] setup db to store model registry (could this be replaced down the line with mlflow?)
- [ ] setup health probe alongside service with tonic-health [docs](https://github.com/hyperium/tonic/tree/master/examples/src/health) 
  - use gRPC health probe in k8s deployment
  - there's also [this](https://github.com/grpc-ecosystem/grpc-health-probe) cli tool
- [ ] minio in deployment for storing model weights


## notes 
- to run a health check first spin up the server with `cargo run --bin server` then `grpc-health-probe -addr="[::1]:50051"`
