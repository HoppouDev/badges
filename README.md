<center>

# ʙᴀᴅɢᴇꜱ

A silly badges API.

</center>

## Development:

```sh
# Test
cargo test

# Preview API locally
wrangler dev
```

## Deploying

```sh
# Deploy to a cloudfare worker
wrangler deploy

# A GitHub token is required for CI functionality
wrangler secret put GITHUB_TOKEN
```

## License

This project is licensed under the MIT License. For badges's license, [LICENSE.md](LICENSE.md).
