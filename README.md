# `repo-birthday` 🎂

Gives you a calendar to let you know when your repos are celebrating their birthdays — and their age.

Just access the URL, log in with GitHub and add your `.ical` URL to your calendar app — or share with your geek friends!

## Contributing

### Environment variables

`repo-birthday` requires some environment variables to access GitHub and to Cloudflare's KV (the storage):

#### Required

| Name | Description |
|---|---|
| `SECRET_KEY` | Used in the encryption/decryption when saving users' GitHub access token into the storage (a random long string will do it) |
| `GITHUB_APP_CLIENT_ID` | GitHub app client ID |
| `GITHUB_APP_SECRET` | GitHub app's secret |
| `CLOUDFLARE_ACCOUNT_ID` | Your Cloudflare's account ID |
| `CLOUDFLARE_API_TOKEN` | Your Cloudflare's API token (must have write permissions to KV) |
| `CLOUDFLARE_KV_NAMESPACE` | The ID of the KV namespace to use for this project |

#### Optional


| Name | Description |
|---|---|
| `PORT` | Which port the web server will listen |
| `DOMAIN` | The domain where your server is running (e.g. `repobirth.day`) |

### Running the server

```console
cargo run
```

### After editing code

```console
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged && cargo check
```
