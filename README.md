# Blog

Throwing this here cause I may use it later.

```rust
let db = Builder::new_remote_replica(root.join("replica.db"), libsql_url, libsql_token)
    .sync_interval(Duration::from_secs(60))
    .build()
    .await?;
```

