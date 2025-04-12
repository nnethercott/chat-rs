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
- [ ] add test actions and branch protection rules 
- [ ] cargo workspace setup
- [ ] re-read docs on streams and futures in rust (see if we can avoid the ReceiverStream pattern)
