# `repo-birthday` 🎂

Tells when your repos are celebrating their birthdays — and their age.

Requires an environment variable `GITHUB_ACCESS_TOKEN` with a GitHub token with ` public_repo` scope.

```console
$ cargo run -- <your GitHub username>
```

For example, for my case, `cargo run -- cuducos`.

> [!WARNING]
> If you have too many repos, maybe the API will complain about too many requests. This is a known bug.
